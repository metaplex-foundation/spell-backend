use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::types::JsonRpcResponse;
use jsonrpc_core::{IoHandler, Params};
use serde::de::DeserializeOwned;
use std::future::Future;

#[derive(Clone)]
pub struct RpcMethodBuilder {
    handler: IoHandler,
    ctx: ArcedAppCtx,
}

#[allow(dead_code)]
impl RpcMethodBuilder {
    pub fn new(ctx: ArcedAppCtx) -> Self {
        Self {
            handler: IoHandler::new(),
            ctx,
        }
    }

    pub fn add_method_without_params<FunctionResult>(
        mut self,
        name: &str,
        endpoint: fn(ArcedAppCtx) -> FunctionResult,
    ) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
    {
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move {
                _params.expect_no_params()?;
                endpoint(cloned_ctx).await
            }
        };

        self.handler.add_method(name, closure);

        self
    }

    pub fn add_method_without_ctx_and_params<FunctionResult>(
        mut self,
        name: &str,
        endpoint: fn() -> FunctionResult,
    ) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
    {
        let closure = move |_params: Params| {
            async move {
                _params.expect_no_params()?;
                endpoint().await
            }
        };

        self.handler.add_method(name, closure);

        self
    }

    pub fn add_method<FunctionResult, RequestParam>(
        mut self,
        name: &str,
        endpoint: fn(RequestParam, ArcedAppCtx) -> FunctionResult,
    ) -> Self
    where
        FunctionResult: Future<Output = JsonRpcResponse> + Sized + Send + 'static,
        RequestParam: DeserializeOwned + Send + 'static,
    {
        let cloned_ctx = self.ctx.clone();

        let closure = move |_params: Params| {
            let cloned_ctx = cloned_ctx.clone();
            async move { endpoint(_params.parse()?, cloned_ctx).await }
        };

        self.handler.add_method(name, closure);

        self
    }

    pub fn build(self) -> IoHandler {
        self.handler
    }
}
