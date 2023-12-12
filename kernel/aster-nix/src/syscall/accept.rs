// SPDX-License-Identifier: MPL-2.0

use super::{SyscallReturn, SYS_ACCEPT};
use crate::{
    fs::{
        file_table::{FdFlags, FileDesc},
        utils::{CreationFlags, StatusFlags},
    },
    log_syscall_entry,
    prelude::*,
    syscall::SYS_ACCEPT4,
    util::net::{get_socket_from_fd, write_socket_addr_to_user},
};

pub fn sys_accept(
    sockfd: FileDesc,
    sockaddr_ptr: Vaddr,
    addrlen_ptr: Vaddr,
) -> Result<SyscallReturn> {
    log_syscall_entry!(SYS_ACCEPT);
    debug!("sockfd = {sockfd}, sockaddr_ptr = 0x{sockaddr_ptr:x}, addrlen_ptr = 0x{addrlen_ptr:x}");

    let fd = do_accept(sockfd, sockaddr_ptr, addrlen_ptr, Flags::empty())?;
    Ok(SyscallReturn::Return(fd as _))
}

pub fn sys_accept4(
    sockfd: FileDesc,
    sockaddr_ptr: Vaddr,
    addrlen_ptr: Vaddr,
    flags: u32,
) -> Result<SyscallReturn> {
    log_syscall_entry!(SYS_ACCEPT4);
    trace!("raw flags = 0x{:x}", flags);
    let flags = Flags::from_bits_truncate(flags);
    debug!(
        "sockfd = {}, sockaddr_ptr = 0x{:x}, addrlen_ptr = 0x{:x}, flags = {:?}",
        sockfd, sockaddr_ptr, addrlen_ptr, flags
    );

    let fd = do_accept(sockfd, sockaddr_ptr, addrlen_ptr, flags)?;
    Ok(SyscallReturn::Return(fd as _))
}

fn do_accept(
    sockfd: FileDesc,
    sockaddr_ptr: Vaddr,
    addrlen_ptr: Vaddr,
    flags: Flags,
) -> Result<FileDesc> {
    let (connected_socket, socket_addr) = {
        let socket = get_socket_from_fd(sockfd)?;
        socket.accept()?
    };

    if flags.contains(Flags::SOCK_NONBLOCK) {
        connected_socket.set_status_flags(StatusFlags::O_NONBLOCK)?;
    }

    let fd_flags = if flags.contains(Flags::SOCK_CLOEXEC) {
        FdFlags::CLOEXEC
    } else {
        FdFlags::empty()
    };

    if sockaddr_ptr != 0 {
        write_socket_addr_to_user(&socket_addr, sockaddr_ptr, addrlen_ptr)?;
    }

    let fd = {
        let current = current!();
        let mut file_table = current.file_table().lock();
        file_table.insert(connected_socket, fd_flags)
    };

    Ok(fd)
}

bitflags! {
    struct Flags: u32 {
        const SOCK_NONBLOCK = NONBLOCK;
        const SOCK_CLOEXEC = CLOEXEC;
    }
}

const NONBLOCK: u32 = StatusFlags::O_NONBLOCK.bits();
const CLOEXEC: u32 = CreationFlags::O_CLOEXEC.bits();
