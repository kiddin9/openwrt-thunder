# nas-xunlei
[![CI](https://github.com/gngpp/nas-xunlei/actions/workflows/CI.yml/badge.svg)](https://github.com/gngpp/nas-xunlei/actions/workflows/CI.yml)
<a href="/LICENSE">
    <img src="https://img.shields.io/github/license/gngpp/nas-xunlei?style=flat">
  </a>
  <a href="https://github.com/gngpp/nas-xunlei/releases">
    <img src="https://img.shields.io/github/release/gngpp/nas-xunlei.svg?style=flat">
  </a><a href="hhttps://github.com/gngpp/nas-xunlei/releases">
    <img src="https://img.shields.io/github/downloads/gngpp/nas-xunlei/total?style=flat&?">
  </a>

nas-xunlei从迅雷群晖套件中提取，用于发行版Linux（支持OpenWrt）的迅雷远程下载程序。仅供测试，测试完请大家自觉删除。

- 只支持**X86_64**/**aarch64**
- 支持glibc/musl
- 内侧邀请码（3H9F7Y6D）

```shell
❯ ./xunlei                   
Synology Nas Thunder runs on Linux

Usage: xunlei [OPTIONS] <COMMAND>

Commands:
  install    Install xunlei
  uninstall  Uninstall xunlei
  launch     Launch xunlei
  help       Print this message or the help of the given subcommand(s)

Options:
  -d, --debug    Enable debug
  -h, --help     Print help
  -V, --version  Print version

```

### Ubuntu(Other Linux)
GitHub [Releases](https://github.com/gngpp/nas-xunlei/releases) 中有预编译的 deb包/rpm包，二进制文件，以Ubuntu为例：
```shell
wget https://github.com/gngpp/nas-xunlei/releases/download/v3.5.2/xunlei_3.5.2_amd64.deb

dpkg -i xunlei_3.5.2_amd64.deb

# 安装和运行迅雷程序
xunlei install
# 停止和卸载迅雷程序
xunlei uninstall
# 如果你的系统不支持systemd，则手动启动
xunlei launch
```

### OpenWrt 路由器
GitHub [Releases](https://github.com/gngpp/nas-xunlei/releases) 中有预编译的 ipk 文件， 目前提供了 aarch64/x86_64 等架构的版本，可以下载后使用 opkg 安装，以 nanopi r4s 为例：

```shell
wget https://github.com/gngpp/nas-xunlei/releases/download/v3.5.2/xunlei_3.5.2-1_aarch64_generic.ipk
wget https://github.com/gngpp/nas-xunlei/releases/download/v3.5.2/luci-app-xunlei_1.0.1_all.ipk
wget https://github.com/gngpp/nas-xunlei/releases/download/v3.5.2/luci-i18n-xunlei-zh-cn_1.0.1-1_all.ipk

opkg install xunlei_3.5.2-1_aarch64_generic.ipk
opkg install luci-app-xunlei_1.0.1_all.ipk
opkg install luci-i18n-xunlei-zh-cn_1.0.1-1_all.ipk
```

### 自行编译

```shell
git clone https://github.com/gngpp/nas-xunlei && cd nas-xunlei

# 默认编译在线安装
cargo build --release && mv target/release/xunlei .

# 完整打包编译安装
bash +x ./unpack.sh && cargo build --release --features embed && mv target/release/xunlei .

# 执行安装
./xunlei install
# 若系统不支持systemctl，则手动启动daemon
./xunlei launch
```

### OpenWrt编译

```shell
cd package
svn co https://github.com/gngpp/nas-xunlei/trunk/openwrt
cd -
make menuconfig # choose LUCI->Applications->Luci-app-xunlei  
make V=s
```

### FQA
 - 如果你是升级了最新版本无法启动/或者装过别人打包的xunlei，请先SSH执行`service xunlei disable && service xunlei enable`
 - 迅雷插件依赖bash，系统需要安装bash