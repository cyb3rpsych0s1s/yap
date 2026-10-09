module Yap

class YapService extends ScriptableService {
    private cb func OnLoad() {
        Life("Service/OnLoad");
        let s = "WaWaWaWa";
        Error(s"Error \(s)");
        Warn(s"Warn \(s)");
        Info(s"Info \(s)");
        Debug(s"Debug \(s)");
        Trace(s"Trace \(s)");
        FTLog(s"FTLog \(s)");
        let system = GameInstance.GetCallbackSystem();
        system.RegisterCallback(n"Session/BeforeStart", this, n"OnSession");
        system.RegisterCallback(n"Session/Start", this, n"OnSession");
        system.RegisterCallback(n"Session/Ready", this, n"OnSession");
        system.RegisterCallback(n"Session/BeforeEnd", this, n"OnSession");
        system.RegisterCallback(n"Session/End", this, n"OnSession");
    }
    private cb func OnInitialize() {
        Life("Service/OnInitialize");
    }
    private cb func OnUninitialize() {
        Life("Service/OnUninitialize");
    }
    private cb func OnReload() {
        Life("Service/OnReload");
    }
    private cb func OnSession(event: ref<GameSessionEvent>) {
        Life(NameToString(event.GetEventName()));
    }
}
class YapSystem extends ScriptableSystem {
    func OnAttach() -> Void {
        Life("System/OnAttach");
    }
    func OnDetach() -> Void {
        Life("System/OnDetach");
    }
}
