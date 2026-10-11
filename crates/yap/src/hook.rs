mod ftlog;
mod logchannel;
mod logger;

pub fn attach_hook(env: &::red4ext_rs::SdkEnv) {
    ftlog::attach_hook(env);
    logchannel::attach_hook(env);
    logger::attach_hook(env);
}
pub fn detach_hook(env: &::red4ext_rs::SdkEnv) {
    logger::detach_hook(env);
    logchannel::detach_hook(env);
    ftlog::detach_hook(env);
}
