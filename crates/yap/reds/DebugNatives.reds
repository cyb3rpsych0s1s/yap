// exist natively, but must be re-declared for the redscript compiler.
// root namespace (no module), debug only: `just ci` strips `Debug*.reds` from releases.
// the natives read `text` as a ref: declaring it `String` crashes the game (null CString)
public native static func LogChannel(channel: CName, text: script_ref<String>) -> Void;
public native static func LogChannelWarning(channel: CName, text: script_ref<String>) -> Void;
public native static func LogChannelError(channel: CName, text: script_ref<String>) -> Void;
