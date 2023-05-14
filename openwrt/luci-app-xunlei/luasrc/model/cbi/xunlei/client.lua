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

o = s:option(Value, "download_path", translate("Default Download Path"), translate("Note: the default storage path, after boot will mount to the bound directory"))
o.default = "/opt/xunlei/downloads"

o = s:option(Value, "mount_bind_download_path", translate("Default Mount Bind Download Path"), translate("Note: Please change the binding download directory you determine at the first startup. It will bind to the real storage path after startup, otherwise it will not be able to change the binding directory after startup"))
o.default = "/xunlei"

return m
