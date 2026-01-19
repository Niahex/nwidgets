use cef::wrapper::message_router::{
    MessageRouterConfig, MessageRouterRendererSide, MessageRouterRendererSideHandlerCallbacks,
    RendererSideRouter,
};
use cef::{
    rc::Rc, Browser, Frame, ImplRenderProcessHandler, ProcessId, ProcessMessage,
    RenderProcessHandler, V8Context, WrapRenderProcessHandler,
};

#[derive(Clone)]
pub struct GpuiRenderProcessHandler {
    message_router: std::sync::Arc<RendererSideRouter>,
}

impl GpuiRenderProcessHandler {
    pub fn new() -> Self {
        let config = MessageRouterConfig::default();
        let message_router = RendererSideRouter::new(config);
        Self { message_router }
    }
}

cef::wrap_render_process_handler! {
    pub struct RenderProcessHandlerWrapper {
        handler: GpuiRenderProcessHandler,
    }

    impl RenderProcessHandler {
        fn on_context_created(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            let browser_owned = browser.as_ref().map(|b| (*b).clone());
            let frame_owned = frame.as_ref().map(|f| (*f).clone());
            let context_owned = context.as_ref().map(|c| (*c).clone());

            self.handler.message_router.on_context_created(
                browser_owned,
                frame_owned,
                context_owned,
            );
        }

        fn on_context_released(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            let browser_owned = browser.as_ref().map(|b| (*b).clone());
            let frame_owned = frame.as_ref().map(|f| (*f).clone());
            let context_owned = context.as_ref().map(|c| (*c).clone());

            self.handler.message_router.on_context_released(
                browser_owned,
                frame_owned,
                context_owned,
            );
        }

        fn on_process_message_received(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> i32 {
            let browser_owned = browser.as_ref().map(|b| (*b).clone());
            let frame_owned = frame.as_ref().map(|f| (*f).clone());
            let message_owned = message.as_ref().map(|m| (*m).clone());

            if self.handler.message_router.on_process_message_received(
                browser_owned,
                frame_owned,
                Some(source_process),
                message_owned,
            ) {
                1
            } else {
                0
            }
        }
    }
}
