# 实验环境准备

## Qemu 安装

可参照官方网站：[Qemu 官网](https://www.qemu.org/download/)

## ArceOS

### Linux make 开发环境

可参照官方网站：[ArceOS Github](https://github.com/arceos-org/arceos)

我们选择在 `APP=examples/httpclient` 的环境下进行测试。

### OSTool 开发环境(实验性)

可支持Linux或Windows下编译和Qemu运行ArceOS，相较于现有make脚本，需要对ArceOS的配置有更深入的了解。

参照 Qemu 官网安装，Windows 需安装 msys2 版 qemu.

安装开发工具

```bash
cargo install ostool
```

进入到ArceOS 项目根目录，运行如下命令

```bash
ostool run qemu
```

我们选择平台 `aarch64-qemu-virt`, 应用 `arceos-httpclient`, 之后就会启动qemu，并运行 ArceOS 的 httpclient 示例。

运行结果如下：

```shell
[  0.084034 axruntime::lang_items:5] panicked at modules\axnet\src\smoltcp_impl\mod.rs:323:25:
invalid IP address: ()
```

这是因为ArceOS的IP地址是写死的，通过编译时的环境变量配置。在项目目录找到`.project.toml`，在 `[compile.env]`下面添加

```text
AX_IP = "10.0.2.15"
AX_GW = "10.0.2.2"
```

指定IP为Qemu的默认IP地址，然后重新运行，可看到成功访问了外部网络。

```shell
[  0.080496 axnet::smoltcp_impl:333]   ether:    52-54-00-12-34-56
[  0.080964 axnet::smoltcp_impl:334]   ip:       10.0.2.15/24
[  0.081316 axnet::smoltcp_impl:335]   gateway:  10.0.2.2
[  0.081565 axhal::platform::aarch64_common::psci:113] Starting CPU 1 ON ...
[  0.082115 axruntime::mp:36] Secondary CPU 1 started.
[  0.082119 axruntime:186] Primary CPU 0 init OK.
[  0.082457 axruntime::mp:46] Secondary CPU 1 init OK.
Hello, simple http client!
dest: 49.12.234.183:80 (49.12.234.183:80)
HTTP/1.1 200 OK
Server: nginx
Date: Mon, 02 Dec 2024 02:52:33 GMT
Content-Type: text/plain
Content-Length: 14
Connection: keep-alive
Access-Control-Allow-Origin: *
Cache-Control: no-cache, no-store, must-revalidate

121.250.51.169
[  0.542744 0 axhal::platform::aarch64_common::psci:96] Shutting down...
```

## 模拟Igb设备

### Make 方式

在linux系统下可使用make方式运行，需修改arceos目录下Makefile文件，更改 Qemu 模拟网卡为 igb。
将`qemu_args-$(NET)`的参数，由`-device virtio-net-$(vdev-suffix),netdev=net0`修改为`-device igb,netdev=net0`。

在arceos目录下运行make命令：
```bash
make A=examples/httpclient PLATFORM=aarch64-qemu-virt LOG=debug SMP=2 NET=y NET_DEV=user run
```

### ostool 方式

修改`.project.toml`，`[compile]`字段中，修改 `features = ["axstd/log-level-debug", "axstd/smp"]`，使输出日志为`debug`级别。

`[qemu]`字段中，修改 `args = "-smp 2 -netdev user,id=net0 -device igb,netdev=net0"`，更改 Qemu 模拟网卡为 igb。

运行输出如下：

```shell
[  0.013761 axmm:60] Initialize virtual memory management...
[  0.023464 axmm:63] kernel address space init OK: AddrSpace {
    va_range: VA:0xffff000000000000..VA:0xfffffffffffff000,
    page_table_root: PA:0x40160000,
}
[  0.024915 axruntime:150] Initialize platform devices...
[  0.025270 axdriver:152] Initialize device drivers...
[  0.025514 axdriver:153]   device model: static
[  0.025977 axdriver::bus::pci:97] PCI 00:00.0: 1b36:0008 (class 06.00, rev 00) Standard
[  0.027063 axdriver::bus::pci:97] PCI 00:01.0: 8086:10c9 (class 02.00, rev 01) Standard
[  0.027742 axdriver::bus::pci:54]   BAR 0: MEM [0x10000000, 0x10020000)
[  0.028057 axdriver::bus::pci:54]   BAR 1: MEM [0x10020000, 0x10040000)
[  0.028699 axdriver::bus::pci:54]   BAR 3: MEM [0x10040000, 0x10044000) 64bit
[  0.076271 axdriver:160] number of NICs: 0
[  0.076550 axnet:43] Initialize network subsystem...
[  0.076906 axruntime::lang_items:5] panicked at modules\axnet\src\lib.rs:45:35:
No NIC device found!
[  0.077666 axhal::platform::aarch64_common::psci:96] Shutting down...
```

其中 PCI 00:01.0 就是Igb网卡，由于我们还没有驱动，所以未发现网卡。

## 参考链接

[1] [Intel IGB Driver](https://github.com/intel/ethernet-linux-igb)
