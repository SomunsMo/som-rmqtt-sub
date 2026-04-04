# Somunsm Rust MQTT Subscriber

一个用 Rust 实现的 MQTT 协议订阅端

## 立项原因

想要小米音箱支持Wol来唤醒局域网内的电脑设备，又不想买个开机卡插在PCIe上，白占用一个通道位置。恰巧家里有个装了OpenWrt的小米路由器，恰巧米家又能接入巴法云。

所以想到在OpenWrt上订阅巴法云的MQTT消息，在OpenWrt上根据消息内容执行预先配置好的shell命令。

### 为什么不用 Python + EthanHome-WOL

本人路由器剩余的存储空间不足4MB，连基础的Python环境都无法安装。

## 运行示例 - OpenWrt(mips)

1. 先下载或自己构建可执行程序
2. 终端所在位置应为“**可执行程序所在目录**”
3. 执行下方命令以部署

```shell
wget -O- https://github.com/SomunsMo/som-rmqtt-sub/raw/master/sh/create%20service.sh | sh
```

若机器无法直接连接Github，建议用其他方式手动下载[脚本](https://github.com/SomunsMo/som-rmqtt-sub/raw/master/sh/create%20service.sh)
和[配置文件](https://github.com/SomunsMo/som-rmqtt-sub/raw/master/config/config.json5)。  
下载完毕后，将**脚本**、**配置文件**、**可执行程序**放置在同一目录下，然后执行下方命令

```shell
chmod +x create_service.sh
./create_service.sh
```

### 编辑配置文件

脚本执行完毕后，需要编辑配置文件中的参数。    
[配置参考模板](https://github.com/SomunsMo/som-rmqtt-sub/blob/master/config/config.json5)

```shell
vi /etc/som-rmqtt-sub/config.json5
```

### 启动程序

确认配置无误后即可启动 MQTT 订阅端

```shell
# 启动程序
/etc/init.d/som-rmqtt-sub start

# 如需开机自启则再执行
/etc/init.d/som-rmqtt-sub enable  
```

## 构建可执行文件（参考）

下面的构建内容仅供参考，在不同平台不同环境构建时，构建的命令也不尽相同。  
**仅对mipsel的构建结果进行了校验，其他平台的构建结果未经验证！**

### MacOS 下构建

| 分类     |                                                 | 版本                                          |
|:-------|:------------------------------------------------|:--------------------------------------------|
| 系统     | MacOS (x86_64)                                  | 15.7.4                                      | 
| 构建工具   | cargo                                           | cargo 1.96.0-nightly (888f67534 2026-03-30) |
| 交叉构建   | [Cross](https://github.com/rust-embedded/cross) | cross 0.2.5 (f86fd03 2026-03-25)            |
| 压缩(可选) | [UPX](https://upx.github.io/)                   | 5.1.1                                       |

```shell
# Windows
cross build --target x86_64-pc-windows-gnu --release -Z build-std=core,std,alloc,panic_abort
cross build --target aarch64-pc-windows-msvc --release -Z build-std=core,std,alloc,panic_abort

# MacOS
cross build --target x86_64-apple-darwin --release -Z build-std=core,std,alloc,panic_abort
cross build --target aarch64-apple-darwin --release -Z build-std=core,std,alloc,panic_abort

# Linux
cross build --target x86_64-unknown-linux-gnu --release -Z build-std=core,std,alloc,panic_abort
cross build --target aarch64-unknown-linux-gnu --release -Z build-std=core,std,alloc,panic_abort
cross build --target armv7-unknown-linux-gnueabihf --release -Z build-std=core,std,alloc,panic_abort

# mipsel
cross build --target mipsel-unknown-linux-musl --release -Z build-std=core,std,alloc,panic_abort
```

#### 压缩

构建之后，体积可能有些大，有些设备存储空间不足，可用UPX进一步压缩。

```shell
# 进入对应架构的 release 目录执行
upx --ultra-brute --no-backup som-rmqtt-sub
```

### Windows 下构建

Windows 下仅举例构建 Windows 的可执行文件

| 分类   |           | 版本                            |
|:-----|:----------|:------------------------------|
| 系统   | Windows11 | 23H2                          |
| 构建工具 | cargo     | 1.94.1 (29ea6fb6a 2026-03-24) |

```shell
cargo run --package som-rmqtt-sub --bin som-rmqtt-sub --release
```

## 报错解决方案

### mipsel 下运行时缺少 "libatomic.so.1" 库

```shell
# 输出的错误信息参考
Error loading shared library libatomic.so.1: No such file or directory (needed by ./som-rmqtt-sub)
Error relocating ./som-rmqtt-sub: __atomic_fetch_add_8: symbol not found
Error relocating ./som-rmqtt-sub: __atomic_load_8: symbol not found
Error relocating ./som-rmqtt-sub: __atomic_store_8: symbol not found
Error relocating ./som-rmqtt-sub: __atomic_fetch_sub_8: symbol not found
Error relocating ./som-rmqtt-sub: __atomic_is_lock_free: symbol not found
```

#### 解决方案

OpenWrt内大概率有这个库，大概率未链接。应使用软连接方式：

```shell
ln -s /lib/libatomic.so.1 /usr/lib/libatomic.so.1
```