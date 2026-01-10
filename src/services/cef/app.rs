use cef::{wrap_app, App, CefString, CommandLine, ImplApp, ImplCommandLine, WrapApp};
use cef::rc::Rc;

wrap_app! {
    pub struct CefApp;

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            if let Some(line) = command_line {
                line.append_switch(Some(&CefString::from("disable-gpu")));
                line.append_switch(Some(&CefString::from("disable-gpu-compositing")));
                line.append_switch(Some(&CefString::from("enable-begin-frame-scheduling")));
            }
        }
    }
}
