local m, s

m = Map("xunlei", translate("Xunlei"))
m.description = translate("<a>NAS Xunlei DSM 7.x Beta Version</a> | <a href=\"https://github.com/gngpp/nas-xunlei\" target=\"_blank\">Project GitHub URL</a>")

m:section(SimpleSection).template = "xunlei/xunlei_status"

s = m:section(TypedSection, "xunlei")
s.addremove = false
s.anonymous = true

o = s:option(Flag, "enabled", translate("Enabled"))
o.rmempty = false

o = s:option(Value, "host", translate("Host"))
o.default = "0.0.0.0"
o.datatype = "ipaddr"

o = s:option(Value, "port", translate("Port"))
o.datatype = "and(port,min(1025))"
o.default = "5055"
o.rmempty = false

o = s:option(Value, "auth_user", translate("Username"))
o = s:option(Value, "auth_password", translate("Password"))
o.password = true

o = s:option(Value, "config_path", translate("Data Storage Path"), translate("Note: Please keep your user data safe"))
o.default = "/opt/xunlei"

o = s:option(Value, "download_path", translate("Download Storage Path"), translate("Download storage path, after starting will be mounted to the thunder download bindings directory"))
o.default = "/opt/xunlei/downloads"

o = s:option(Value, "mount_bind_download_path", translate("Mount Bind Download Path"), translate("The download bindings directory will be mapped to the download storage path after startup, no special changes are required"))
o.default = "/xunlei"

return m
