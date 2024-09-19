use crate::endpoints::types::JsonRpcResponse;
use crate::setup::app_context::ArcedAppCtx;
use jsonrpc_core::{IoHandler, Params};
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::future::Future;
use tracing::info;

/// `RpcMethodRegistrar` is a structure designed for registering JSON-RPC methods with an application context.
/// It allows methods with different parameters to be registered
///     and automatically determines the method names based on the function passed.
///
/// Unfortunately, to achieve this API, we had to limit ourselves in some cases.
/// 1) We cannot pass different structures into the state of our application; we can only use a single one.
/// 2) Additionally, we can only pass parameters that implement the `Serialize` and `Deserialize` traits.
/// 3) Also, if we register a method with parameters and context,
///  they must be passed in a strict order, with the parameter coming first and the context second.
/// 4) Also, all methods must return a `JsonRpcResponse` type.
///
/// For examples of such methods implementation, refer to the `src/endpoints` folder.
///
/// Usage example:
/// ```ignore
/// #[derive(Serialize, Deserialize)]
/// struct RequestParam {
///     id: u32,
///     name: String,
/// }
///
/// async fn get_user(param: RequestParam, ctx: ArcedAppCtx) -> JsonRpcResponse {
///     let res = ctx.user_service.get_user(param.id, param.name).await?;
///     Ok(json!(res))
/// }
///
/// async fn get_status() -> JsonRpcResponse {
///     Ok(json!("Service is running".to_string()))
/// }
///
/// async fn get_random_number(ctx: ArcedAppCtx) -> JsonRpcResponse {
///     let res = ctx.randomizer.get_number();
///     Ok(json!(res))
///  }
///
/// async fn main() {
///     let app_ctx = AppCtx::new(/* args */).await.arced();
///
///     let io_handler = RpcMethodRegistrar::using_ctx(app_ctx.clone())
///         .method(get_user)
///         .method_without_params(get_random_number)
///         .method_without_ctx_and_params(get_status)
///         .add_alias("getUser", "get_user")
///         .add_alias("getRandomNumber", "get_random_number")
///         .add_alias("getStatus", "get_status")
///         .finish();
///
///     ServerBuilder::new(handler)
///         .health_api(("/health", "health"))
///         .start_http(&(/* args */))
///         .inspect_err(|e| error!("Failed to start http: {e}."))?
///         .wait();
/// }
/// ```
#[derive(Clone)]
pub struct RpcMethodRegistrar {
    handler: IoHandler,
    ctx: ArcedAppCtx,
}

#[allow(dead_code)]
impl RpcMethodRegistrar {
    /// Creates a new instance of `RpcMethodRegistrar` using the provided application context (State).
    pub fn using_ctx(ctx: ArcedAppCtx) -> Self {
        info!("Registration of RPC methods has started.");
        Self { handler: IoHandler::new(), ctx }
    }

    /// Registers a method that takes no parameters other than the application context.
    pub fn method_without_params<FunctionResult>(mut self, endpoint: fn(ArcedAppCtx) -> FunctionResult) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
    {
        let method_name = Self::get_endpoint_name(&endpoint);
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move {
                _params.expect_no_params()?;
                endpoint(cloned_ctx).await
            }
        };

        self.handler.add_method(&method_name, closure);

        info!("Added method: '{method_name}'.");

        self
    }

    /// Registers a method that takes no parameters and does not use the application context.
    pub fn method_without_ctx_and_params<FunctionResult>(mut self, endpoint: fn() -> FunctionResult) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
    {
        let method_name = Self::get_endpoint_name(&endpoint);

        let closure = move |_params: Params| async move {
            _params.expect_no_params()?;
            endpoint().await
        };

        self.handler.add_method(&method_name, closure);

        info!("Added method: '{method_name}'.");

        self
    }

    /// Registers a method that takes a parameter and the application context.
    pub fn method<FunctionResult, RequestParam>(
        mut self,
        endpoint: fn(RequestParam, ArcedAppCtx) -> FunctionResult,
    ) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
        RequestParam: DeserializeOwned + Send + 'static,
    {
        let method_name = Self::get_endpoint_name(&endpoint);
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move { endpoint(_params.parse()?, cloned_ctx).await }
        };

        self.handler.add_method(&method_name, closure);

        info!("Added method: '{method_name}'.");

        self
    }

    /// Adds an alias for a registered method.
    pub fn add_alias(mut self, alias: &str, for_method: &str) -> Self {
        info!("Adding alias '{alias}' for method '{for_method}'.");
        self.handler.add_alias(alias, for_method);
        self
    }

    /// Completes the registration of methods and returns the handler containing all registered methods.
    pub fn finish(self) -> IoHandler {
        info!("Registration of RPC methods has ended.");
        self.handler
    }

    /// Retrieves the method name based on the function’s path.
    fn get_endpoint_name<T>(endpoint: &T) -> String {
        let full_path = Self::get_type_full_path(endpoint);
        let mut parts = full_path.split("::").collect::<Vec<&str>>();
        parts.pop();
        parts
            .pop()
            .unwrap_or_else(|| panic!("Couldn't extract function name from '{full_path}'"))
            .to_string()
    }

    fn get_type_full_path<T>(_: T) -> String {
        type_name::<T>().to_owned()
    }

    #[cfg(test)]
    fn method_dispatch<FunctionResult, RequestParam>(
        endpoint: fn(RequestParam, ArcedAppCtx) -> FunctionResult,
    ) -> fn(RequestParam, ArcedAppCtx) -> FunctionResult
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
        RequestParam: DeserializeOwned + Send + 'static,
    {
        endpoint
    }
}

#[cfg(test)]
mod test {
    use crate::endpoints::get_nft::get_asset;
    use crate::setup::method_registrar::RpcMethodRegistrar;

    #[test]
    fn test_extraction_of_endpoint_name() {
        let endpoint = get_asset;
        let endpoint = RpcMethodRegistrar::method_dispatch(endpoint);

        let full_path = RpcMethodRegistrar::get_type_full_path(endpoint);
        dbg!(&full_path);
        let actual_res = RpcMethodRegistrar::get_endpoint_name(&endpoint);
        let expected_res = "get_asset";

        dbg!(&actual_res);

        assert_eq!(actual_res, expected_res);
    }
}
