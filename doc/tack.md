# 开发思路

## 1. ArceOS 添加驱动

驱动管理位于`axdriver`模块，先添加`igb`驱动的`feature`:

```toml
# modules/axdriver/Cargo.toml

igb  = ["net", "dep:axalloc"]
```

网卡驱动依赖axalloc模块来获取堆内存支持，net 模块则提供了网络协议栈支持。

修改axdriver的build.rs脚本，添加igb驱动的编译选项：

```rust
const NET_DEV_FEATURES: &[&str] = &["igb", "ixgbe", "virtio-net"];
```

在`modules/axdriver/src/drivers.rs`中，添加igb网卡驱动的加载代码，可参照ixgbe网卡驱动的加载代码：

```rust
cfg_if::cfg_if! {
    if #[cfg(net_dev = "ixgbe")] {
        //...
    }
}
```

可根据vid、pid来判断是否为igb网卡，并加载igb网卡驱动。

在 `modules/axdriver/src/macros.rs` 中添加igb的注册代码：

```rust
#[cfg(net_dev = "igb")]
{
    type $drv_type = crate::drivers::IgbDriver;
    $code
}
```

在 `api/axfeat/Cargo.toml` 中添加igb的`feature`，使应用可以选择是否加载igb网卡驱动。

```toml
driver-igb = ["axdriver?/igb"]
```

在`ulib/axstd/Cargo.toml`中添加：

```toml
driver-igb = ["axfeat/driver-igb"]
```

之后就可以在make中通过`FEATURES=driver-igb`来选择加载igb网卡驱动了。

```bash
make A=examples/httpclient PLATFORM=aarch64-qemu-virt LOG=debug SMP=2 NET=y FEATURES=driver-igb NET_DEV=user run
```

ostool 则通过添加`features` `"axstd/driver-igb"`。

## 2. 网卡驱动逻辑

可参照手册 [Intel 82576 Gigabit Ethernet Controller Datasheet](https://www.intel.com/content/dam/www/public/us/en/documents/datasheets/82576eb-gigabit-ethernet-controller-datasheet.pdf) 第4.5章节，理解网卡初始化有哪些步骤，具体实现可参照igb源码。

需理解ring的概念，主要在7.1、7.2章节。
