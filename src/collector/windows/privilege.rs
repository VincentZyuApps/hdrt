pub(crate) fn is_elevated() -> bool {
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::BOOL;
    use windows_sys::Win32::Security::{
        AllocateAndInitializeSid, CheckTokenMembership, FreeSid, SECURITY_NT_AUTHORITY,
    };

    const SECURITY_BUILTIN_DOMAIN_RID: u32 = 0x0000_0020;
    const DOMAIN_ALIAS_RID_ADMINS: u32 = 0x0000_0220;

    unsafe {
        let mut admin_group = null_mut();
        let allocated = AllocateAndInitializeSid(
            &SECURITY_NT_AUTHORITY,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut admin_group,
        );
        if allocated == 0 {
            return false;
        }

        let mut is_member: BOOL = 0;
        let checked = CheckTokenMembership(null_mut(), admin_group, &mut is_member);
        FreeSid(admin_group);

        checked != 0 && is_member != 0
    }
}
