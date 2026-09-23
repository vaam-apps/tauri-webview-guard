# The plugin class and its @Command methods are reached by reflection from
# Tauri's PluginManager, so R8 cannot see any reference to them and would strip
# them from a release build. The symptom is "No command engine found", only in
# release, only on a minified app — and for this plugin that means the guard
# silently never runs.
-keep class app.vaam.webviewguard.** { *; }
