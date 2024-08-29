use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::types::JsonRpcResponse;
use jsonrpc_core::{IoHandler, Params};
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::future::Future;
use tracing::info;

#[derive(Clone)]
pub struct RpcMethodRegistrar {
    handler: IoHandler,
    ctx: ArcedAppCtx,
}

#[allow(dead_code)]
impl RpcMethodRegistrar {
    pub fn new(ctx: ArcedAppCtx) -> Self {
        info!("Registration of RPC methods has started.");
        Self {
            handler: IoHandler::new(),
            ctx,
        }
    }

    pub fn method_without_params<FunctionResult>(
        mut self,
        endpoint: fn(ArcedAppCtx) -> FunctionResult,
    ) -> Self
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

    pub fn method_without_ctx_and_params<FunctionResult>(
        mut self,
        endpoint: fn() -> FunctionResult,
    ) -> Self
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

    pub fn add_alias(mut self, alias: &str, for_method: &str) -> Self {
        info!("Adding alias '{alias}' for method '{for_method}'.");
        self.handler.add_alias(alias, for_method);
        self
    }

    pub fn finish(self) -> IoHandler {
        info!("Registration of RPC methods has ended.");
        self.handler
    }

    /// Split the path by "::" and collect into a vector.
    /// Then remove the last part.
    /// Get the penultimate part.
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
    use crate::config::method_builder::RpcMethodRegistrar;
    use crate::endpoints::get_nft::get_asset;

    #[test]
    fn test() {
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
