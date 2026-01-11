use futures::channel::mpsc::UnboundedSender;
use gpui::ClipboardItem;

/// Generic clipboard interception script for CEF browsers
/// Intercepts all clipboard operations and sends them via document.title
pub const CLIPBOARD_SCRIPT: &str = r#"
(function(){
const send=t=>{if(!t)return;const o=document.title;document.title='__NWIDGETS_COPY__:'+t;setTimeout(()=>{if(document.title.startsWith('__NWIDGETS_COPY__'))document.title=o},500)};
document.addEventListener('copy',e=>{const t=e.clipboardData?.getData('text/plain')||window.getSelection()?.toString();if(t)send(t)});
if(navigator.clipboard){
const w=navigator.clipboard.writeText?.bind(navigator.clipboard);
if(w)navigator.clipboard.writeText=t=>{send(t);return w(t).catch(()=>Promise.resolve())};
const wr=navigator.clipboard.write?.bind(navigator.clipboard);
if(wr)navigator.clipboard.write=d=>{return Promise.resolve(d).then(items=>{for(const item of items){item.getType?.('text/plain')?.then(b=>b?.text?.())?.then(t=>send(t))?.catch(()=>{})}return wr(items)}).catch(()=>Promise.resolve())};
}
const exec=document.execCommand.bind(document);
document.execCommand=(cmd,...args)=>{if(cmd==='copy'){const t=window.getSelection()?.toString();if(t)send(t)}return exec(cmd,...args)};
})();
"#;

pub const CLIPBOARD_PREFIX: &str = "__NWIDGETS_COPY__:";

/// Check if a title string contains clipboard data and extract it
pub fn extract_clipboard_data(title: &str) -> Option<&str> {
    title.strip_prefix(CLIPBOARD_PREFIX)
}

/// Spawn a task to handle clipboard updates from CEF
pub fn spawn_clipboard_handler<V: 'static>(
    cx: &mut gpui::Context<'_, V>,
    clipboard_rx: futures::channel::mpsc::UnboundedReceiver<String>,
) {
    cx.spawn(|_, cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let mut clipboard_rx = clipboard_rx;
            while let Some(text) = futures::StreamExt::next(&mut clipboard_rx).await {
                let _ = cx.update(|cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                });
            }
        }
    })
    .detach();
}

/// Create a clipboard channel for CEF communication
pub fn create_clipboard_channel() -> (
    UnboundedSender<String>,
    futures::channel::mpsc::UnboundedReceiver<String>,
) {
    futures::channel::mpsc::unbounded()
}
