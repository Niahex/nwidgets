use gpui::{ClipboardEntry, ClipboardItem};
use cef::{Browser, CefString, ImplBrowser, ImplFrame};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn inject_clipboard_to_cef(browser: &Browser, clipboard_item: &ClipboardItem) {
    let mut image_script = None;
    let mut text_script = None;

    for entry in clipboard_item.entries() {
        match entry {
            ClipboardEntry::Image(image) => {
                let mime = image.format.mime_type();
                let b64 = STANDARD.encode(&image.bytes);
                // JS to write image and dispatch paste event
                let js = format!(r#"(
                    async () => {{
                        try {{
                            const b64 = "{}";
                            const mime = "{}";
                            const byteCharacters = atob(b64);
                            const byteNumbers = new Array(byteCharacters.length);
                            for (let i = 0; i < byteCharacters.length; i++) {{
                                byteNumbers[i] = byteCharacters.charCodeAt(i);
                            }}
                            const byteArray = new Uint8Array(byteNumbers);
                            const blob = new Blob([byteArray], {{type: mime}});
                            
                            // 1. Write to clipboard
                            await navigator.clipboard.write([
                                new ClipboardItem({{
                                    [mime]: blob
                                }})
                            ]);

                            // 2. Dispatch paste event
                            const dataTransfer = new DataTransfer();
                            const file = new File([blob], "pasted-image.png", {{ type: mime }});
                            dataTransfer.items.add(file);
                            
                            const pasteEvent = new ClipboardEvent('paste', {{
                                bubbles: true,
                                cancelable: true,
                                clipboardData: dataTransfer
                            }});
                            document.activeElement.dispatchEvent(pasteEvent);

                        }} catch (e) {{
                            console.error("Failed to inject image:", e);
                        }}
                    }})();
                "# , b64, mime);
                image_script = Some(js);
                break; 
            }
            ClipboardEntry::String(s) => {
                if text_script.is_none() {
                     let escaped = s.text()
                        .replace('\\', "\\\\")
                        .replace('`', "\\`")
                        .replace("${ ", "\\${ ");
                     
                     // JS to write text and insert it
                     text_script = Some(format!(r#"(
                        async () => {{
                            try {{
                                const text = `{}`;
                                await navigator.clipboard.writeText(text);
                                document.execCommand('insertText', false, text);
                            }} catch (e) {{
                                console.error("Failed to inject text:", e);
                            }}
                        }})();
                     "#, escaped));
                }
            }
            _ => {}
        }
    }

    let script = image_script.or(text_script);

    if let Some(script) = script {
        if let Some(frame) = browser.main_frame() {
            frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("clipboard_inject")), 0);
        }
    }
}