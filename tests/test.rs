#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bare_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bare_test::{driver::device_tree::get_device_tree, fdt::PciSpace, mem::mmu::iomap, println};
use core::ptr::NonNull;
use igb_driver::{IgbDevice, IgbHal, MemPool, NicDevice, PhysAddr as IgbPhysAddr,IgbNetBuf};
use log::{debug, info};
use pcie::*;

use alloc::alloc::{alloc, dealloc};
use core::mem;
use core::time::Duration;
use core::arch::asm;

bare_test::test_setup!();

#[test_case]
fn it_works1() {
    println!("test1... ");
    assert_eq!(1, 1);
}

const MEM_POOL: usize = 4096;
const MEM_POOL_ENTRY_SIZE: usize = 2048;
const QS: usize = 16;
const QN: u16 = 2;

#[test_case]
fn test_igb() {
    // 获取基地址和长度
    let (base, len) = get_igb();
    let addr = iomap(base.into(), len);
    debug!("igb start,{:?}", addr);
    //分配 MemPool
    let mem_pool = MemPool::allocate::<TestIgbHalImpl>(MEM_POOL, MEM_POOL_ENTRY_SIZE).unwrap();
    //初始化设置
    let inner =
        IgbDevice::<TestIgbHalImpl, QS>::init(addr.addr().into(), len, QN, QN, &mem_pool).unwrap();
    // 如何测试发包  https://github.com/ixy-languages/ixy.rs/blob/master/examples/generator.rs    
    // https://github.com/lispking/rust-e1000-driver/blob/main/src/e1000/e1000.rs
    // 目前没搞定
    //let tx_buf= IgbNetBuf::alloc(pool, size);
    //inner.send(0, tx_buf);
    
}

/// 只用于测试的 IgbHal 实现
pub struct TestIgbHalImpl;

unsafe impl IgbHal for TestIgbHalImpl {
    fn dma_alloc(size: usize) -> (IgbPhysAddr, NonNull<u8>) {
        // Allocate memory using `alloc`
        let layout = core::alloc::Layout::from_size_align(size, mem::align_of::<u8>()).unwrap();
        let ptr = unsafe { alloc(layout) as *mut u8 };
        if ptr.is_null() {
            panic!("Memory allocation failed");
        }
        let non_null_ptr = unsafe { NonNull::new_unchecked(ptr) };

        // Return the physical address and the pointer
        (ptr as IgbPhysAddr, non_null_ptr)
    }

    unsafe fn dma_dealloc(_paddr: IgbPhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        // Deallocate memory using `dealloc`
        let layout = core::alloc::Layout::from_size_align(size, mem::align_of::<u8>()).unwrap();
        unsafe {
            dealloc(vaddr.as_ptr(), layout);
        }
        0 // Return 0 for success
    }

    unsafe fn mmio_phys_to_virt(paddr: IgbPhysAddr, _size: usize) -> NonNull<u8> {
        // In real implementations, this would map the MMIO region
        // For now, assume a no-op for the sake of example.
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> IgbPhysAddr {
        // In real implementations, this would return the physical address
        // For now, assume a no-op for the sake of example.
        vaddr.as_ptr() as IgbPhysAddr
    }

    /// 这个函数只支持在arm64 架构下测试,借助chatgpt 编写
    fn wait_until(duration: Duration) -> Result<(), &'static str> {
        // 获取计时器频率（每秒计数次数）
        let freq = Self::get_timer_frequency();

        // 将 Duration 转换为计时器周期数（tick 数）
        let target_ticks = duration.as_secs() * freq + duration.subsec_nanos() as u64 * freq / 1_000_000_000;

        // 获取当前计时器计数
        let start_ticks = Self::get_timer_count();

        // 等待直到经过足够的周期
        while Self::get_timer_count() - start_ticks < target_ticks {
            // 可选：执行 no-op 操作以节省 CPU 时间
            unsafe {
                asm!("nop");
            }
        }
    
        Ok(())
    }
}

impl TestIgbHalImpl {
    // 获取当前的计时器计数
    fn get_timer_count() -> u64 {
        let count: u64;
        unsafe {
            asm!(
                "mrs {}, cntpct_el0", // 从 cntrpct_el0 寄存器读取计时器计数
                out(reg) count,
            );
        }
        count
    }

    // 获取计时器的频率
    fn get_timer_frequency() -> u64 {
        let freq: u64;
        unsafe {
            asm!(
                "mrs {}, cntfrq_el0", // 从 cntfrq_el0 寄存器读取计时器频率
                out(reg) freq,
            );
        }
        freq
    }
}


fn get_igb() -> (usize, usize) {
    let fdt = get_device_tree().unwrap();
    let pcie = fdt
        .find_compatible(&["pci-host-ecam-generic"])
        .next()
        .unwrap()
        .into_pci()
        .unwrap();

    let mut pcie_regs = alloc::vec![];

    println!("pcie: {}", pcie.node.name);

    for reg in pcie.node.reg().unwrap() {
        println!("pcie reg: {:#x}", reg.address);
        pcie_regs.push(iomap((reg.address as usize).into(), reg.size.unwrap()));
    }

    let mut bar_alloc = SimpleBarAllocator::default();

    for range in pcie.ranges().unwrap() {
        info!("pcie range: {:?}", range);

        match range.space {
            PciSpace::Memory32 => bar_alloc.set_mem32(range.cpu_address as _, range.size as _),
            PciSpace::Memory64 => bar_alloc.set_mem64(range.cpu_address, range.size),
            _ => {}
        }
    }

    let base_vaddr = pcie_regs[0];

    info!("Init PCIE @{:?}", base_vaddr);

    let mut root = RootComplexGeneric::new(base_vaddr);

    for elem in root.enumerate(None, Some(bar_alloc)) {
        debug!("PCI {}", elem);

        if let Header::Endpoint(ep) = elem.header {
            ep.update_command(elem.root, |cmd| {
                cmd | CommandRegister::IO_ENABLE
                    | CommandRegister::MEMORY_ENABLE
                    | CommandRegister::BUS_MASTER_ENABLE
            });

            if ep.vendor_id == 0x8086 && ep.device_id == 0x10C9 {
                let bar_addr;
                let bar_size;
                match ep.bar {
                    pcie::BarVec::Memory32(bar_vec_t) => {
                        let bar0 = bar_vec_t[0].as_ref().unwrap();
                        bar_addr = bar0.address as usize;
                        bar_size = bar0.size as usize;
                    }
                    pcie::BarVec::Memory64(bar_vec_t) => {
                        let bar0 = bar_vec_t[0].as_ref().unwrap();
                        bar_addr = bar0.address as usize;
                        bar_size = bar0.size as usize;
                    }
                    pcie::BarVec::Io(_bar_vec_t) => todo!(),
                };

                println!("bar0: {:#x}", bar_addr);

                //let addr = iomap(bar_addr.into(), bar_size);
                //let igb = Igb::new(addr);
                // return igb;
                return (bar_addr, bar_size);
            }
        }
    }
    panic!("no igb found");
}
