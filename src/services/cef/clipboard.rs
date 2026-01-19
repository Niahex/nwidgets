use futures::channel::mpsc::UnboundedSender;
use gpui::{ClipboardItem, Image, ImageFormat};

#[derive(Debug, Clone)]
pub enum ClipboardData {
    Text(String),
    Image { data: Vec<u8>, format: ImageFormat },
}

/// Enhanced clipboard script that intercepts Clipboard API calls
/// Sends clipboard data via window.cefQuery to sync with system clipboard
pub const CLIPBOARD_SCRIPT: &str = r#"
(function(){
if(!window.cefQuery)return;
const send=(type,data)=>{
window.cefQuery({
request:JSON.stringify({type:'clipboard',data:{type:type,content:data}}),
persistent:false,
onSuccess:()=>{},
onFailure:()=>{}
});
};
document.addEventListener('copy',e=>{
setTimeout(()=>{
const sel=window.getSelection();
if(sel&&!sel.isCollapsed){
send('text',sel.toString());
}
},0);
},true);
const origWrite=navigator.clipboard?.write?.bind(navigator.clipboard);
if(origWrite){
navigator.clipboard.write=async(data)=>{
try{
const items=await Promise.resolve(data);
for(const item of items){
if(item.types?.includes('text/plain')){
const blob=await item.getType('text/plain');
const text=await blob.text();
send('text',text);
}else if(item.types?.includes('image/png')){
const blob=await item.getType('image/png');
const reader=new FileReader();
reader.onload=()=>{
const base64=reader.result.split(',')[1];
send('image',base64);
};
reader.readAsDataURL(blob);
}
}
}catch(e){}
return origWrite(data);
};
}
const origWriteText=navigator.clipboard?.writeText?.bind(navigator.clipboard);
if(origWriteText){
navigator.clipboard.writeText=t=>{
send('text',t);
return origWriteText(t);
};
}
})();
"#;

/// Check if a message contains clipboard data and extract it
pub fn extract_clipboard_from_message(message: &str) -> Option<ClipboardData> {
    // Parse JSON message from cefQuery
    let parsed: serde_json::Value = serde_json::from_str(message).ok()?;
    
    if parsed.get("type")?.as_str()? != "clipboard" {
        return None;
    }
    
    let data = parsed.get("data")?;
    let data_type = data.get("type")?.as_str()?;
    let content = data.get("content")?.as_str()?;
    
    match data_type {
        "text" => Some(ClipboardData::Text(content.to_string())),
        "image" => {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD.decode(content).ok()?;
            
            let format = if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                ImageFormat::Png
            } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                ImageFormat::Jpeg
            } else if bytes.starts_with(b"WEBP") || (bytes.len() > 12 && &bytes[8..12] == b"WEBP") {
                ImageFormat::Webp
            } else {
                ImageFormat::Png
            };
            
            Some(ClipboardData::Image {
                data: bytes,
                format,
            })
        }
        _ => None,
    }
}

/// Spawn a task to handle clipboard updates from CEF
pub fn spawn_clipboard_handler<V: 'static>(
    cx: &mut gpui::Context<'_, V>,
    clipboard_rx: futures::channel::mpsc::UnboundedReceiver<ClipboardData>,
) {
    cx.spawn(|_, cx: &mut gpui::AsyncApp| {
        let cx = cx.clone();
        async move {
            let mut clipboard_rx = clipboard_rx;
            while let Some(data) = futures::StreamExt::next(&mut clipboard_rx).await {
                cx.update(|cx| {
                    match data {
                        ClipboardData::Text(text) => {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                        }
                        ClipboardData::Image { data, format } => {
                            let image = Image::from_bytes(format, data);
                            cx.write_to_clipboard(ClipboardItem::new_image(&image));
                        }
                    }
                });
            }
        }
    })
    .detach();
}

/// Create a clipboard channel for CEF communication
pub fn create_clipboard_channel() -> (
    UnboundedSender<ClipboardData>,
    futures::channel::mpsc::UnboundedReceiver<ClipboardData>,
) {
    futures::channel::mpsc::unbounded()
}
