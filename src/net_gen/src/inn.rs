// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// automatically generated by tools/bindgen.sh

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case,
    clippy::ptr_as_ptr
)]

pub const IP_TOS: u32 = 1;
pub const IP_TTL: u32 = 2;
pub const IP_HDRINCL: u32 = 3;
pub const IP_OPTIONS: u32 = 4;
pub const IP_ROUTER_ALERT: u32 = 5;
pub const IP_RECVOPTS: u32 = 6;
pub const IP_RETOPTS: u32 = 7;
pub const IP_PKTINFO: u32 = 8;
pub const IP_PKTOPTIONS: u32 = 9;
pub const IP_MTU_DISCOVER: u32 = 10;
pub const IP_RECVERR: u32 = 11;
pub const IP_RECVTTL: u32 = 12;
pub const IP_RECVTOS: u32 = 13;
pub const IP_MTU: u32 = 14;
pub const IP_FREEBIND: u32 = 15;
pub const IP_IPSEC_POLICY: u32 = 16;
pub const IP_XFRM_POLICY: u32 = 17;
pub const IP_PASSSEC: u32 = 18;
pub const IP_TRANSPARENT: u32 = 19;
pub const IP_RECVRETOPTS: u32 = 7;
pub const IP_ORIGDSTADDR: u32 = 20;
pub const IP_RECVORIGDSTADDR: u32 = 20;
pub const IP_MINTTL: u32 = 21;
pub const IP_NODEFRAG: u32 = 22;
pub const IP_CHECKSUM: u32 = 23;
pub const IP_BIND_ADDRESS_NO_PORT: u32 = 24;
pub const IP_RECVFRAGSIZE: u32 = 25;
pub const IP_PMTUDISC_DONT: u32 = 0;
pub const IP_PMTUDISC_WANT: u32 = 1;
pub const IP_PMTUDISC_DO: u32 = 2;
pub const IP_PMTUDISC_PROBE: u32 = 3;
pub const IP_PMTUDISC_INTERFACE: u32 = 4;
pub const IP_PMTUDISC_OMIT: u32 = 5;
pub const IP_MULTICAST_IF: u32 = 32;
pub const IP_MULTICAST_TTL: u32 = 33;
pub const IP_MULTICAST_LOOP: u32 = 34;
pub const IP_ADD_MEMBERSHIP: u32 = 35;
pub const IP_DROP_MEMBERSHIP: u32 = 36;
pub const IP_UNBLOCK_SOURCE: u32 = 37;
pub const IP_BLOCK_SOURCE: u32 = 38;
pub const IP_ADD_SOURCE_MEMBERSHIP: u32 = 39;
pub const IP_DROP_SOURCE_MEMBERSHIP: u32 = 40;
pub const IP_MSFILTER: u32 = 41;
pub const IP_MULTICAST_ALL: u32 = 49;
pub const IP_UNICAST_IF: u32 = 50;
pub const IP_DEFAULT_MULTICAST_TTL: u32 = 1;
pub const IP_DEFAULT_MULTICAST_LOOP: u32 = 1;
pub const IN_CLASSA_NET: u32 = 4278190080;
pub const IN_CLASSA_NSHIFT: u32 = 24;
pub const IN_CLASSA_HOST: u32 = 16777215;
pub const IN_CLASSA_MAX: u32 = 128;
pub const IN_CLASSB_NET: u32 = 4294901760;
pub const IN_CLASSB_NSHIFT: u32 = 16;
pub const IN_CLASSB_HOST: u32 = 65535;
pub const IN_CLASSB_MAX: u32 = 65536;
pub const IN_CLASSC_NET: u32 = 4294967040;
pub const IN_CLASSC_NSHIFT: u32 = 8;
pub const IN_CLASSC_HOST: u32 = 255;
pub const IN_MULTICAST_NET: u32 = 4026531840;
pub const IN_LOOPBACKNET: u32 = 127;
pub const INADDR_LOOPBACK: u32 = 2130706433;
pub const INADDR_UNSPEC_GROUP: u32 = 3758096384;
pub const INADDR_ALLHOSTS_GROUP: u32 = 3758096385;
pub const INADDR_ALLRTRS_GROUP: u32 = 3758096386;
pub const INADDR_MAX_LOCAL_GROUP: u32 = 3758096639;
pub type __be32 = u32;
pub const IPPROTO_IP: _bindgen_ty_1 = 0;
pub const IPPROTO_ICMP: _bindgen_ty_1 = 1;
pub const IPPROTO_IGMP: _bindgen_ty_1 = 2;
pub const IPPROTO_IPIP: _bindgen_ty_1 = 4;
pub const IPPROTO_TCP: _bindgen_ty_1 = 6;
pub const IPPROTO_EGP: _bindgen_ty_1 = 8;
pub const IPPROTO_PUP: _bindgen_ty_1 = 12;
pub const IPPROTO_UDP: _bindgen_ty_1 = 17;
pub const IPPROTO_IDP: _bindgen_ty_1 = 22;
pub const IPPROTO_TP: _bindgen_ty_1 = 29;
pub const IPPROTO_DCCP: _bindgen_ty_1 = 33;
pub const IPPROTO_IPV6: _bindgen_ty_1 = 41;
pub const IPPROTO_RSVP: _bindgen_ty_1 = 46;
pub const IPPROTO_GRE: _bindgen_ty_1 = 47;
pub const IPPROTO_ESP: _bindgen_ty_1 = 50;
pub const IPPROTO_AH: _bindgen_ty_1 = 51;
pub const IPPROTO_MTP: _bindgen_ty_1 = 92;
pub const IPPROTO_BEETPH: _bindgen_ty_1 = 94;
pub const IPPROTO_ENCAP: _bindgen_ty_1 = 98;
pub const IPPROTO_PIM: _bindgen_ty_1 = 103;
pub const IPPROTO_COMP: _bindgen_ty_1 = 108;
pub const IPPROTO_SCTP: _bindgen_ty_1 = 132;
pub const IPPROTO_UDPLITE: _bindgen_ty_1 = 136;
pub const IPPROTO_MPLS: _bindgen_ty_1 = 137;
pub const IPPROTO_RAW: _bindgen_ty_1 = 255;
pub const IPPROTO_MAX: _bindgen_ty_1 = 256;
pub type _bindgen_ty_1 = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct in_addr {
    pub s_addr: __be32,
}
#[test]
fn bindgen_test_layout_in_addr() {
    assert_eq!(
        ::std::mem::size_of::<in_addr>(),
        4usize,
        concat!("Size of: ", stringify!(in_addr))
    );
    assert_eq!(
        ::std::mem::align_of::<in_addr>(),
        4usize,
        concat!("Alignment of ", stringify!(in_addr))
    );
    fn test_field_s_addr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<in_addr>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).s_addr) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(in_addr),
                "::",
                stringify!(s_addr)
            )
        );
    }
    test_field_s_addr();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
}
#[test]
fn bindgen_test_layout_ip_mreq() {
    assert_eq!(
        ::std::mem::size_of::<ip_mreq>(),
        8usize,
        concat!("Size of: ", stringify!(ip_mreq))
    );
    assert_eq!(
        ::std::mem::align_of::<ip_mreq>(),
        4usize,
        concat!("Alignment of ", stringify!(ip_mreq))
    );
    fn test_field_imr_multiaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreq>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_multiaddr) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreq),
                "::",
                stringify!(imr_multiaddr)
            )
        );
    }
    test_field_imr_multiaddr();
    fn test_field_imr_interface() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreq>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_interface) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreq),
                "::",
                stringify!(imr_interface)
            )
        );
    }
    test_field_imr_interface();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ip_mreqn {
    pub imr_multiaddr: in_addr,
    pub imr_address: in_addr,
    pub imr_ifindex: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_ip_mreqn() {
    assert_eq!(
        ::std::mem::size_of::<ip_mreqn>(),
        12usize,
        concat!("Size of: ", stringify!(ip_mreqn))
    );
    assert_eq!(
        ::std::mem::align_of::<ip_mreqn>(),
        4usize,
        concat!("Alignment of ", stringify!(ip_mreqn))
    );
    fn test_field_imr_multiaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreqn>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_multiaddr) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreqn),
                "::",
                stringify!(imr_multiaddr)
            )
        );
    }
    test_field_imr_multiaddr();
    fn test_field_imr_address() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreqn>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_address) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreqn),
                "::",
                stringify!(imr_address)
            )
        );
    }
    test_field_imr_address();
    fn test_field_imr_ifindex() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreqn>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_ifindex) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreqn),
                "::",
                stringify!(imr_ifindex)
            )
        );
    }
    test_field_imr_ifindex();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ip_mreq_source {
    pub imr_multiaddr: __be32,
    pub imr_interface: __be32,
    pub imr_sourceaddr: __be32,
}
#[test]
fn bindgen_test_layout_ip_mreq_source() {
    assert_eq!(
        ::std::mem::size_of::<ip_mreq_source>(),
        12usize,
        concat!("Size of: ", stringify!(ip_mreq_source))
    );
    assert_eq!(
        ::std::mem::align_of::<ip_mreq_source>(),
        4usize,
        concat!("Alignment of ", stringify!(ip_mreq_source))
    );
    fn test_field_imr_multiaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreq_source>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_multiaddr) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreq_source),
                "::",
                stringify!(imr_multiaddr)
            )
        );
    }
    test_field_imr_multiaddr();
    fn test_field_imr_interface() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreq_source>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_interface) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreq_source),
                "::",
                stringify!(imr_interface)
            )
        );
    }
    test_field_imr_interface();
    fn test_field_imr_sourceaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_mreq_source>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imr_sourceaddr) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_mreq_source),
                "::",
                stringify!(imr_sourceaddr)
            )
        );
    }
    test_field_imr_sourceaddr();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct ip_msfilter {
    pub imsf_multiaddr: __be32,
    pub imsf_interface: __be32,
    pub imsf_fmode: u32,
    pub imsf_numsrc: u32,
    pub imsf_slist: [__be32; 1usize],
}
#[test]
fn bindgen_test_layout_ip_msfilter() {
    assert_eq!(
        ::std::mem::size_of::<ip_msfilter>(),
        20usize,
        concat!("Size of: ", stringify!(ip_msfilter))
    );
    assert_eq!(
        ::std::mem::align_of::<ip_msfilter>(),
        4usize,
        concat!("Alignment of ", stringify!(ip_msfilter))
    );
    fn test_field_imsf_multiaddr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_msfilter>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imsf_multiaddr) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_msfilter),
                "::",
                stringify!(imsf_multiaddr)
            )
        );
    }
    test_field_imsf_multiaddr();
    fn test_field_imsf_interface() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_msfilter>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imsf_interface) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_msfilter),
                "::",
                stringify!(imsf_interface)
            )
        );
    }
    test_field_imsf_interface();
    fn test_field_imsf_fmode() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_msfilter>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imsf_fmode) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_msfilter),
                "::",
                stringify!(imsf_fmode)
            )
        );
    }
    test_field_imsf_fmode();
    fn test_field_imsf_numsrc() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_msfilter>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imsf_numsrc) as usize - ptr as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_msfilter),
                "::",
                stringify!(imsf_numsrc)
            )
        );
    }
    test_field_imsf_numsrc();
    fn test_field_imsf_slist() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<ip_msfilter>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).imsf_slist) as usize - ptr as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(ip_msfilter),
                "::",
                stringify!(imsf_slist)
            )
        );
    }
    test_field_imsf_slist();
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct in_pktinfo {
    pub ipi_ifindex: ::std::os::raw::c_int,
    pub ipi_spec_dst: in_addr,
    pub ipi_addr: in_addr,
}
#[test]
fn bindgen_test_layout_in_pktinfo() {
    assert_eq!(
        ::std::mem::size_of::<in_pktinfo>(),
        12usize,
        concat!("Size of: ", stringify!(in_pktinfo))
    );
    assert_eq!(
        ::std::mem::align_of::<in_pktinfo>(),
        4usize,
        concat!("Alignment of ", stringify!(in_pktinfo))
    );
    fn test_field_ipi_ifindex() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<in_pktinfo>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).ipi_ifindex) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(in_pktinfo),
                "::",
                stringify!(ipi_ifindex)
            )
        );
    }
    test_field_ipi_ifindex();
    fn test_field_ipi_spec_dst() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<in_pktinfo>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).ipi_spec_dst) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(in_pktinfo),
                "::",
                stringify!(ipi_spec_dst)
            )
        );
    }
    test_field_ipi_spec_dst();
    fn test_field_ipi_addr() {
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<in_pktinfo>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).ipi_addr) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(in_pktinfo),
                "::",
                stringify!(ipi_addr)
            )
        );
    }
    test_field_ipi_addr();
}
