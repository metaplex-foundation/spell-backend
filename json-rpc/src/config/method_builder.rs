use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::types::JsonRpcResponse;
use jsonrpc_core::{IoHandler, Params};
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::future::Future;

#[derive(Clone)]
pub struct RpcMethodRegistrar {
    handler: IoHandler,
    ctx: ArcedAppCtx,
}

#[allow(dead_code)]
impl RpcMethodRegistrar {
    pub fn new(ctx: ArcedAppCtx) -> Self {
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
        let method_name = Self::get_fn_name(endpoint).as_str();
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move {
                _params.expect_no_params()?;
                endpoint(cloned_ctx).await
            }
        };

        self.handler.add_method(method_name, closure);

        self
    }

    pub fn method_without_ctx_and_params<FunctionResult>(
        mut self,
        endpoint: fn() -> FunctionResult,
    ) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
    {
        let method_name = Self::get_fn_name(endpoint).as_str();

        let closure = move |_params: Params| async move {
            _params.expect_no_params()?;
            endpoint().await
        };

        self.handler.add_method(method_name, closure);

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
        let method_name = Self::get_fn_name(endpoint).as_str();
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move { endpoint(_params.parse()?, cloned_ctx).await }
        };

        self.handler.add_method(method_name, closure);

        self
    }

    pub fn add_alias(mut self, alias: &str, for_method: &str) -> Self {
        self.handler.add_alias(alias, for_method);
        self
    }

    pub fn finish(self) -> IoHandler {
        self.handler
    }

    fn get_fn_name<T>(_: T) -> String {
        let full_type_name = type_name::<T>();
        full_type_name
            .split("::")
            .last()
            .unwrap_or_else(|_| panic!("Couldn't extract function name of '{full_type_name}'."))
            .to_owned()
    }
}
