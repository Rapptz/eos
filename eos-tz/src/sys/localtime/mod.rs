#[cfg_attr(target_family = "windows", path = "windows.rs")]
#[cfg_attr(target_family = "unix", path = "unix.rs")]
mod imp;

#[cfg(all(not(target_family = "windows"), not(target_family = "unix")))]
compile_error!("The platform you're compiling for is unfortunately unsupported");

pub(crate) use imp::LocalTime;
