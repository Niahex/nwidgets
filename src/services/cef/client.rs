use cef::{wrap_client, Client, ImplClient, WrapClient};
use cef::rc::Rc;

wrap_client! {
    pub struct CefClient;

    impl Client {}
}
