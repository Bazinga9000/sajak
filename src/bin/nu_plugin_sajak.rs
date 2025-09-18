use nu_plugin::{serve_plugin, MsgPackSerializer};
use sajak::nu::plugin::SajakPlugin;

fn main() {
    serve_plugin(&SajakPlugin::new(), MsgPackSerializer)
}
