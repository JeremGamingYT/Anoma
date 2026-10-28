# Launcher commands

<!-- Generated from launcher/src/cli.rs by cli_tools::markdown(); a test fails when this file is out of date. Update it with: ANOMA_WRITE_DOCS=1 cargo test --manifest-path launcher/Cargo.toml the_launcher_reference -->

Every command of `anoma-launcher`, and the tool `anoma-launcher mcp` offers for it. A tool's arguments are the command's, in snake_case. A tool runs the command with `--json` and returns what it printed. The tools that change the player's things for good need `confirm: true`. The tools that send something or reach other people say "only when the player asks". The game's own tools, `game_*`, are in [anoma-ctl-commands.md](anoma-ctl-commands.md). With No AI on, `mcp` and `agent-config` are hidden and refuse; every other command stays. See [agent-control.md](agent-control.md#no-ai).

## `anoma-launcher versions`

List Anoma releases, or the installed versions.

    anoma-launcher versions

- `--installed`: Only the versions installed here (no network)

MCP: `versions`.

## `anoma-launcher install`

Download and install a release (a tag such as v2026.10.13, or "latest").

    anoma-launcher install <TAG>

- `TAG`

MCP: `install`.

## `anoma-launcher remove`

Remove an installed version.

    anoma-launcher remove <TAG>

- `TAG`

MCP: `remove`, needs `confirm: true`.

## `anoma-launcher prepare`

Prepare a version's resources from your own Minecraft install, or from Mojang.

    anoma-launcher prepare

- `--minecraft`: The game folder of the official launcher, Prism Launcher, MultiMC, the Modrinth App, ATLauncher or GDLauncher (default: the launcher's, then every launcher's default)
- `--download`: Download Minecraft from Mojang for the signed-in account that owns it, instead of using an install

MCP: `prepare`.

## `anoma-launcher play`

Start the client (needs a signed-in Microsoft account that owns Minecraft: Java Edition).

    anoma-launcher play

- `--instance`: The instance to play (its name or id; default: the selected one)
- `--world`: Open or create this world (a folder under saves) at once
- `--creative`: Creative mode for a new world
- `--join`: Join a friend's world with its invite code
- `--server`: Join this server at once: an address, or a server of the list by name or position
- `--arg`: An extra argument for the game, passed through as-is (repeatable), for example --server-thread

MCP: `play`, then the `game_*` tools.

## `anoma-launcher server`

Run a dedicated server here; type commands into its console.

    anoma-launcher server

- `--dir`: The server's folder (default: the one the launcher uses)
- `--accept-eula`: Agree to the Minecraft EULA (https://aka.ms/MinecraftEULA)

MCP: `server`, `server_command`, `server_log`, `server_stop`.

## `anoma-launcher notes`

Show a release's patch notes (default: the newest).

    anoma-launcher notes [TAG]

- `TAG`

MCP: `notes`.

## `anoma-launcher links`

Show the Discord, sponsor and website links.

    anoma-launcher links

MCP: `links`.

## `anoma-launcher account list`

The signed-in accounts and which one plays.

    anoma-launcher account list

MCP: `account_list`.

## `anoma-launcher account sign-in`

Sign in with a Microsoft account in your browser (device code). Play needs one that owns Minecraft: Java Edition.

    anoma-launcher account sign-in

MCP: `account_sign_in`.

## `anoma-launcher account sign-out`

Sign out: the account's tokens leave the keychain.

    anoma-launcher account sign-out <WHO>

- `WHO`: Its name or UUID

MCP: `account_sign_out`, needs `confirm: true`.

## `anoma-launcher account select`

Play as this account, or as one of your offline names.

    anoma-launcher account select <WHO>

- `WHO`

MCP: `account_select`.

## `anoma-launcher account offline`

Play offline (LAN, offline-mode servers) beside a signed-in account that owns the game: as NAME (kept to switch back to; 3 to 16 letters, digits or _), or as the name the game made up.

    anoma-launcher account offline [NAME]

- `NAME`

MCP: `account_offline`.

## `anoma-launcher account forget-offline`

Forget one of your offline names.

    anoma-launcher account forget-offline <NAME>

- `NAME`

MCP: `account_forget_offline`.

## `anoma-launcher account skin`

Show or set an offline name's skin and cape: a 64x64 (or 64x32) skin PNG, a 64x32 cape PNG, the model. Kept by the launcher and given to the game when that name plays.

    anoma-launcher account skin <NAME> [FILE]

- `NAME`
- `FILE`: The skin's PNG
- `--model`: classic or slim arms
- `--cape`: The cape's PNG
- `--no-skin`: Back to the default skin
- `--no-cape`: Take the cape off

MCP: `account_skin`.

## `anoma-launcher settings`

Show the launcher's settings, or set one (the Settings page): selected-version TAG, minecraft-dir PATH, server-dir PATH ("default" resets a folder), install-new-versions on|off, opening-animation on|off, proxy http://host:port ("default" removes it), while-playing stay|minimize|hide (what the window does while the game runs), language system|en|fr|ru (the window's language; system follows the computer's), accent violet|blue|teal|green|amber|red|pink, background PATH (a picture behind Home; "default" shows Anoma's landscape).

    anoma-launcher settings [KEY] [VALUE]

- `KEY` (selected-version, minecraft-dir, server-dir, install-new-versions, opening-animation, proxy, while-playing, language, accent, background)
- `VALUE`: The new value (with KEY)

MCP: `settings`.

## `anoma-launcher game-args`

Show or set the extra arguments the game starts with (for example --server-thread); quotes group a word with spaces in it.

    anoma-launcher game-args [ARGS]

- `ARGS`: Omit to show the current arguments

MCP: `game_args`.

## `anoma-launcher share-log`

Upload an instance's latest.log to mclo.gs (a paste service for Minecraft logs) and print its link, for a bug report: only when the player asks. The home folder, the user name and the account names are taken out first; the link expires after 90 days unread.

    anoma-launcher share-log

- `--instance`: The instance (default: the selected one)

MCP: `share_log`, only when the player asks.

## `anoma-launcher unshare-log`

Delete a log shared with share-log: the "<id>:<token>" it printed.

    anoma-launcher unshare-log <SHARED>

- `SHARED`

MCP: `unshare_log`, needs `confirm: true`.

## `anoma-launcher logs`

Print the end of the game's log (logs/latest.log in the game folder), or its newest crash report.

    anoma-launcher logs

- `--lines`: How many lines of latest.log
- `--crash`: The newest crash report instead

MCP: `logs`.

## `anoma-launcher update`

Look for a newer Anoma release, and install it.

    anoma-launcher update

- `--check`: Only report what is available

MCP: `update`.

## `anoma-launcher desktop-entry`

Linux: put Anoma in the applications menu (an entry and the icon, under $XDG_DATA_HOME), or take it out with --remove.

    anoma-launcher desktop-entry

- `--remove`: Remove the entry and the icon, and stop adding them back

MCP: `desktop_entry`.

## `anoma-launcher data status`

What is allowed, the install ID and how many items wait (the default).

    anoma-launcher data status

MCP: `data_status`.

## `anoma-launcher data allow`

Turn kinds on: performance, crashes, bugs, benchmarks, or all.

    anoma-launcher data allow <KINDS>...

- `KINDS`

MCP: `data_allow`, only when the player asks.

## `anoma-launcher data deny`

Turn kinds off: performance, crashes, bugs, benchmarks, or all.

    anoma-launcher data deny <KINDS>...

- `KINDS`

MCP: `data_deny`.

## `anoma-launcher data show`

Print exactly what would be sent.

    anoma-launcher data show

MCP: `data_show`.

## `anoma-launcher data delete`

Delete everything sent from here, and what waits, and take a new install ID.

    anoma-launcher data delete

MCP: `data_delete`, needs `confirm: true`.

## `anoma-launcher data send`

Send what is waiting now.

    anoma-launcher data send

MCP: `data_send`, only when the player asks.

## `anoma-launcher data bug`

Send a bug report you write; it is printed first, as it would leave.

    anoma-launcher data bug <TEXT>

- `TEXT`
- `--log`: Add the end of the game's log (scrubbed, without chat)
- `--yes`: Send without asking

MCP: `data_bug`, only when the player asks.

## `anoma-launcher data benchmark`

Upload a benchmark report (docs/benchmark-report.md); it is printed first.

    anoma-launcher data benchmark <FILE>

- `FILE`
- `--yes`

MCP: `data_benchmark`, only when the player asks.

## `anoma-launcher instance list`

The instances, each with its game folder (* = the one Play starts).

    anoma-launcher instance list

MCP: `instance_list`.

## `anoma-launcher instance create`

Make an instance: empty but for your options and name, or a copy of another one's worlds, options and packs.

    anoma-launcher instance create <NAME>

- `NAME`
- `--copy`: The instance to copy (its name or id)

MCP: `instance_create`.

## `anoma-launcher instance rename`

Rename an instance (its folder stays).

    anoma-launcher instance rename <INSTANCE> <NAME>

- `INSTANCE`
- `NAME`

MCP: `instance_rename`.

## `anoma-launcher instance icon`

Set an instance's icon: a block or item name from your prepared files (grass_block), a colour (#3fa7d6), or none.

    anoma-launcher instance icon <INSTANCE> <ICON>

- `INSTANCE`
- `ICON`

MCP: `instance_icon`.

## `anoma-launcher instance group`

Put an instance in a sidebar group, or take it out with no group.

    anoma-launcher instance group <INSTANCE> [GROUP]

- `INSTANCE`
- `GROUP`

MCP: `instance_group`.

## `anoma-launcher instance select`

Make Play start this instance.

    anoma-launcher instance select <INSTANCE>

- `INSTANCE`

MCP: `instance_select`.

## `anoma-launcher instance version`

Keep an instance on one installed Anoma version, or "latest" to follow updates again.

    anoma-launcher instance version <INSTANCE> <VERSION>

- `INSTANCE`

MCP: `instance_version`.

## `anoma-launcher instance export`

Export an instance (or a port) to share: its settings, packs, Anoma mods and the worlds named, as a .mrpack (resource packs from Modrinth listed by download) or a zip. Never logs, screenshots or account data; worlds go without their per-player files.

    anoma-launcher instance export [INSTANCE]

- `INSTANCE`: The instance (default: the selected one)
- `--format` (mrpack, zip)
- `--world`: A world to put in it (its name or folder); repeat for more
- `--out`: Where to write it (default: "<name>.mrpack" in the current folder)

MCP: `instance_export`.

## `anoma-launcher instance launch`

Show or set what runs around an instance's game: a shell command before it (the game starts only if it succeeds), a wrapper it runs through (`nice -n 5`), and environment variables ("NAME=value NAME2=value"). An empty value clears one. On Linux, also GameMode, MangoHud and the dedicated graphics card, when the computer has them.

    anoma-launcher instance launch <INSTANCE>

- `INSTANCE`
- `--pre-launch`
- `--wrapper`
- `--env`
- `--gamemode`: Linux: run it through GameMode (gamemoderun), on or off
- `--mangohud`: Linux: MangoHud's overlay, on or off
- `--dedicated-gpu`: Linux: the dedicated graphics card (DRI_PRIME, or NVIDIA's offload), on or off

MCP: `instance_launch`, needs `confirm: true`.

## `anoma-launcher instance shortcut`

Put a shortcut on the desktop that plays an instance (`play --instance`): a .desktop file (also in the applications menu) on Linux, an app on macOS, a .lnk on Windows.

    anoma-launcher instance shortcut [INSTANCE]

- `INSTANCE`: The instance (default: the selected one)
- `--remove`: Move the instance's shortcuts to the trash instead

MCP: `instance_shortcut`.

## `anoma-launcher instance import`

Import what a file is, as dropping it on the window does: a shared instance (a .mrpack without mod loaders, such as `instance export` writes, or a game folder zip) becomes a new instance; a Java modpack goes to the Modpacks import; a resource or shader pack into the instance (default: the selected one).

    anoma-launcher instance import <FILE>

- `FILE`
- `--instance`: The instance a pack goes into

MCP: `instance_import`.

## `anoma-launcher worlds list`

The worlds: name, mode, version, last played and size (the default).

    anoma-launcher worlds list

- `--instance`: The instance (default: the selected one)

MCP: `worlds_list`.

## `anoma-launcher worlds backup`

Zip a world into the instance's backups/ folder, as the game's Make Backup does.

    anoma-launcher worlds backup <WORLD>

- `WORLD`: Its folder or name
- `--instance`

MCP: `worlds_backup`.

## `anoma-launcher worlds copy`

Copy a world beside it, as "<name> (1)".

    anoma-launcher worlds copy <WORLD>

- `WORLD`
- `--instance`

MCP: `worlds_copy`.

## `anoma-launcher worlds delete`

Move a world to the system's trash.

    anoma-launcher worlds delete <WORLD>

- `WORLD`
- `--instance`

MCP: `worlds_delete`, needs `confirm: true`.

## `anoma-launcher worlds datapack-list`

A world's data packs, and which are on.

    anoma-launcher worlds datapack-list <WORLD>

- `WORLD`
- `--instance`

MCP: `worlds_datapack_list`.

## `anoma-launcher worlds datapack-add`

Copy a data pack zip into a world (the game turns it on when the world opens).

    anoma-launcher worlds datapack-add <WORLD> <FILE>

- `WORLD`
- `FILE`
- `--instance`

MCP: `worlds_datapack_add`.

## `anoma-launcher worlds datapack-remove`

Move a world's data pack to the system's trash.

    anoma-launcher worlds datapack-remove <WORLD> <PACK>

- `WORLD`
- `PACK`: Its file name, with or without .zip
- `--instance`

MCP: `worlds_datapack_remove`, needs `confirm: true`.

## `anoma-launcher packs list`

The resource and shader packs, and which are on (the default).

    anoma-launcher packs list

- `--kind` (resource, shader): Only this kind
- `--instance`: The instance (default: the selected one)

MCP: `packs_list`.

## `anoma-launcher packs add`

Copy a pack file in: a resource pack .zip, a shader pack .zip or a .hshader.

    anoma-launcher packs add <FILE>

- `FILE`
- `--instance`

MCP: `packs_add`.

## `anoma-launcher packs on`

Turn a pack on (a resource pack goes on top; a shader pack replaces the one that was on).

    anoma-launcher packs on <PACK>

- `PACK`: Its file name or title
- `--kind` (resource, shader)
- `--instance`

MCP: `packs_on`.

## `anoma-launcher packs off`

Turn a pack off.

    anoma-launcher packs off <PACK>

- `PACK`
- `--kind` (resource, shader)
- `--instance`

MCP: `packs_off`.

## `anoma-launcher packs remove`

Turn a pack off and move it to the system's trash.

    anoma-launcher packs remove <PACK>

- `PACK`
- `--kind` (resource, shader)
- `--instance`

MCP: `packs_remove`, needs `confirm: true`.

## `anoma-launcher screenshots list`

The screenshots, the newest first, with their files (the default).

    anoma-launcher screenshots list

- `--instance`: The instance (default: the selected one)

MCP: `screenshots_list`.

## `anoma-launcher screenshots delete`

Move a screenshot to the system's trash.

    anoma-launcher screenshots delete <SCREENSHOT>

- `SCREENSHOT`: Its file name, with or without .png
- `--instance`

MCP: `screenshots_delete`, needs `confirm: true`.

## `anoma-launcher multiplayer list`

The servers of the game's Multiplayer screen (the default).

    anoma-launcher multiplayer list

- `--instance`: The instance (default: the selected one)

MCP: `multiplayer_list`.

## `anoma-launcher multiplayer add`

Add a server to the list.

    anoma-launcher multiplayer add <ADDRESS>

- `ADDRESS`: Its address, as in the game's Add Server (host or host:port)
- `--name`: Its name in the list (default: Minecraft Server)
- `--instance`

MCP: `multiplayer_add`.

## `anoma-launcher multiplayer remove`

Remove a server from the list: its position from 1, name or address.

    anoma-launcher multiplayer remove <SERVER>

- `SERVER`
- `--instance`

MCP: `multiplayer_remove`, needs `confirm: true`.

## `anoma-launcher modpack list`

The imported packs and their coverage (the default).

    anoma-launcher modpack list

MCP: `modpack_list`.

## `anoma-launcher modpack import`

Import a .mrpack, a CurseForge zip or an instance folder, and write its coverage report.

    anoma-launcher modpack import <PATH>

- `PATH`

MCP: `modpack_import`.

## `anoma-launcher modpack port`

Get the jars of what this computer can port and carry them over.

    anoma-launcher modpack port <PACK>

- `PACK`: The pack's folder name, as list shows it
- `--from`: A folder with your own jars (an installed instance, a downloads folder)

MCP: `modpack_port`.

## `anoma-launcher modpack play`

Play a ported pack as its own profile.

    anoma-launcher modpack play <PACK>

- `PACK`

MCP: `modpack_play`.

## `anoma-launcher modpack remove`

Remove an imported pack: its jars, ports and the profile's saves.

    anoma-launcher modpack remove <PACK>

- `PACK`

MCP: `modpack_remove`, needs `confirm: true`.

## `anoma-launcher mods list`

What the browser installed in an instance (the default).

    anoma-launcher mods list

- `--instance`: The instance (default: the selected one)

MCP: `mods_list`.

## `anoma-launcher mods search`

Search for packs Anoma loads.

    anoma-launcher mods search [TEXT]...

- `TEXT`
- `--kind` (mod, resource, modpack, shader, datapack)
- `--everything`: Also what was made for other versions or runs only partly, each marked
- `--source` (modrinth, curseforge)

MCP: `mods_search`.

## `anoma-launcher mods check`

Say how well a Java mod runs in Anoma: its jar is downloaded (checksum checked), carried over and read; the answer is kept.

    anoma-launcher mods check <PROJECT>

- `PROJECT`
- `--everything`: Allow a version made for another Minecraft version (1.21 and later)
- `--source` (modrinth, curseforge)

MCP: `mods_check`.

## `anoma-launcher mods install`

Install a pack and what it needs into an instance, by its id or slug (checksums checked); a Java mod is ported into anoma-mods and turned on; a modpack is imported as an instance of its own; a data pack goes into the world `--world` names.

    anoma-launcher mods install <PROJECT>

- `PROJECT`
- `--world`: Install the project's data pack into this world of the instance (its name or folder)
- `--instance`: The instance (default: the selected one)
- `--everything`: Allow a version made for another Minecraft version
- `--source` (modrinth, curseforge)

MCP: `mods_install`.

## `anoma-launcher mods update`

Update what the browser installed (or one pack).

    anoma-launcher mods update [PROJECT]

- `PROJECT`
- `--check`: Only say what is newer
- `--instance`: The instance (default: the selected one)

MCP: `mods_update`.

## `anoma-launcher mods remove`

Remove a pack the browser installed.

    anoma-launcher mods remove <PROJECT>

- `PROJECT`
- `--instance`: The instance (default: the selected one)

MCP: `mods_remove`, needs `confirm: true`.

## `anoma-launcher mods enable`

Turn on a mod the browser installed (anoma-mods.txt, as the game's Options > Mods does).

    anoma-launcher mods enable <PROJECT>

- `PROJECT`
- `--instance`: The instance (default: the selected one)

MCP: `mods_enable`.

## `anoma-launcher mods disable`

Turn off a mod the browser installed; it stays installed.

    anoma-launcher mods disable <PROJECT>

- `PROJECT`
- `--instance`: The instance (default: the selected one)

MCP: `mods_disable`.

## `anoma-launcher servers list`

The servers, their folder and whether they run.

    anoma-launcher servers list

MCP: `servers_list`.

## `anoma-launcher servers create`

Create a server; with no options it is a new world, survival, normal, 10 players, friends only.

    anoma-launcher servers create

- `--name`
- `--seed`: The new world's seed (default: random)
- `--copy-world`: Start from a copy of this world folder (one of your saves)
- `--mode` (survival, creative, adventure, spectator)
- `--difficulty` (peaceful, easy, normal, hard)
- `--max-players`
- `--open`: Let anyone in (no whitelist)
- `--check-accounts`: Check players' Minecraft accounts (they must sign in with Microsoft)
- `--player`: Who may join (repeatable); the first becomes an operator
- `--accept-eula`: Agree to the Minecraft EULA (https://aka.ms/MinecraftEULA)

MCP: `servers_create`, only when the player asks.

## `anoma-launcher servers set-up`

Copy an installed, prepared version's programs and game files for the servers (again).

    anoma-launcher servers set-up

MCP: `servers_set_up`.

## `anoma-launcher servers run`

Run a anoma-server command on a server, for example: servers run "My server" -- status.

    anoma-launcher servers run <SERVER> -- <ARGS>...

- `SERVER`: The server's name or folder
- `ARGS`

MCP: `servers_run`.

## `anoma-launcher servers forget`

Take a server off the list (its folder stays).

    anoma-launcher servers forget <SERVER>

- `SERVER`

MCP: `servers_forget`, needs `confirm: true`.

## `anoma-launcher mcp`

Serve the Model Context Protocol on stdin and stdout, for AI agents (refused while No AI is on).

    anoma-launcher mcp

MCP: none: this command is the MCP server.

## `anoma-launcher agent-config`

How to connect an AI agent (Claude Code, Codex, Cursor, any MCP client) to `mcp`: the config to paste and the command that adds it (refused while No AI is on).

    anoma-launcher agent-config

- `--client` (claude-code, codex, cursor, generic): Only this client
- `--test`: Start the MCP server as a client would and say whether it answered

MCP: none: it prints how to connect a client to `mcp`.

## `anoma-launcher no-ai`

Show or set No AI: on hides and turns off every AI feature (the companion, agent control, anoma-ctl and both MCP servers); off, the default, makes them available.

    anoma-launcher no-ai [STATE]

- `STATE` (on, off): Omit to show the current state

MCP: none: the player's switch.

## `anoma-launcher benchmark`

Measure the launcher: starts (the first is the cold one), page switching, idle CPU and memory, the release list fetch; writes a benchmark report to benchmarks/ in its data folder.

    anoma-launcher benchmark

- `--repeats`: How many times to start it

MCP: `benchmark`.
