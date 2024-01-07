# thunder
[![CI](https://github.com/gngpp/thunder/actions/workflows/CI.yml/badge.svg)](https://github.com/gngpp/thunder/actions/workflows/CI.yml)
<a href="/LICENSE">
    <img src="https://img.shields.io/github/license/gngpp/thunder?style=flat">
  </a>
  <a href="https://github.com/gngpp/thunder/releases">
    <img src="https://img.shields.io/github/release/gngpp/thunder.svg?style=flat">
  </a><a href="hhttps://github.com/gngpp/thunder/releases">
    <img src="https://img.shields.io/github/downloads/gngpp/xunlei/total?style=flat&?">
  </a>
  [![Docker Image](https://img.shields.io/docker/pulls/gngpp/xunlei.svg)](https://hub.docker.com/r/gngpp/xunlei/)

thunder从迅雷群晖套件中提取，用于发行版Linux（支持OpenWrt/Alpine/Docker）的迅雷远程下载服务。仅供测试，测试完请自觉删除。

- 支持X86_64/aarch64
- 支持glibc/musl
- 支持更改下载目录
- 支持面板认证
- 支持以特定用户安装(UID/GID)
- Docker镜像最小压缩（40MB左右）
- 支持插件：NAS小星（pcdn），测速插件
- 内侧邀请码（3H9F7Y6D/迅雷牛通），内侧码申请快速通道：https://t.cn/A6fhraWZ

> 默认Web访问端口5055

```shell
❯ ./thunder                   
Synology NAS thunder run on Linux

Usage: thunder
       thunder <COMMAND>

Commands:
  install    Install thunder
  uninstall  Uninstall thunder
  run        Run thunder
  start      Start thunder daemon
  stop       Stop thunder daemon
  status     Show the Http server daemon process
  log        Show the Http server daemon log
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Ubuntu(Other Linux)

GitHub [Releases](https://github.com/gngpp/thunder/releases) 中有预编译的 deb包，二进制文件，以Ubuntu为例：

```shell
wget https://github.com/gngpp/thunder/releases/download/v1.0.1/thunder_1.0.1_amd64.deb

dpkg -i thunder_1.0.1_amd64.deb

# 安装迅雷，默认在线下载安装，如果需要设置更多参数请带上`-h`，查看说明
thunder install

# 安装迅雷以指定spk包安装，如果需要设置更多参数请带上`-h`，查看说明
thunder install /root/nasxunlei-DSM7-x86_64.spk

# 卸载迅雷
thunder uninstall

# 前台运行迅雷，如果需要设置更多参数请带上`-h`，查看说明
thunder run 

# 后台运行迅雷，如果需要设置更多参数请带上`-h`，查看说明
thunder start

# 停止运行迅雷
thunder stop

# 查看运行状态
thunder status

# 查看运行日志
thunder log
```

### 自行编译

```shell
git clone https://github.com/gngpp/thunder && cd thunder

cargo build --release && mv target/release/thunder .
```


### FQA
 - 当前大重构，`OpenWrt` / `Docker` 后续再完善支持
 - musl运行库的操作系统，若已存在glibc运行库，那么会优先兼容选择使用操作系统运行库环境（避免对系统其他软件依赖冲突，可能会缺依赖，自行补全）
 - 指定运行LD加载库或压缩目前无法做到（二进制带签名），需要逆向打patch
 - 插件依赖bash，系统需要安装bash
