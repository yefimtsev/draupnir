# Installs draupnir for the current Windows user. No admin rights needed.
#
#   irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1 | iex
#
# Without a prompt (agents, scripts): set DRAUPNIR_VAULT_ROOT to the vault folder first.
# Uninstall (keeps the config, the vault and the installed skills):
#
#   & ([scriptblock]::Create((irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1))) -Uninstall

param([switch]$Uninstall)

# Child scope: `irm | iex` runs in the caller's session, so nothing set here may leak into it.
& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'  # the progress bar slows Invoke-WebRequest down a lot on PowerShell 5.1
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $Asset = 'https://github.com/yefimtsev/draupnir/releases/latest/download/draupnir-x86_64-pc-windows-msvc.exe'
    $Dir = Join-Path $env:LOCALAPPDATA 'Programs\draupnir'
    $Exe = Join-Path $Dir 'draupnir.exe'

    # The user PATH is read and written raw through the registry: the [Environment] API expands
    # %VARS% on read and stores REG_SZ on write, which would bake every %USERPROFILE% entry into a fixed path.
    $EnvKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    $PathEntries = @(([string]$EnvKey.GetValue('Path', '', 'DoNotExpandEnvironmentNames')) -split ';' | Where-Object { $_ })

    function Save-UserPath($entries) {
        $EnvKey.SetValue('Path', ($entries -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
        # Touching a variable through the API broadcasts WM_SETTINGCHANGE, so new terminals see the change.
        [Environment]::SetEnvironmentVariable('DRAUPNIR_INSTALL_PING', '1', 'User')
        [Environment]::SetEnvironmentVariable('DRAUPNIR_INSTALL_PING', $null, 'User')
    }

    if ($Uninstall) {
        if (Test-Path $Dir) { Remove-Item $Dir -Recurse -Force }
        Save-UserPath ($PathEntries | Where-Object { $_ -ne $Dir })
        Write-Host "Removed $Dir and its PATH entry. The config, the vault and the skills in ~\.claude\skills are left in place."
        return
    }

    New-Item -ItemType Directory -Force $Dir | Out-Null
    Write-Host "Downloading $Asset"
    Invoke-WebRequest $Asset -OutFile $Exe -UseBasicParsing

    if ($PathEntries -notcontains $Dir) {
        Save-UserPath ($PathEntries + $Dir)
        Write-Host "Added $Dir to your user PATH."
    }
    $env:Path = "$env:Path;$Dir"

    # Same lookup as the CLI: $HOME when set (Git Bash sets it), else the user profile; %APPDATA% second.
    $HomeDir = if ($env:HOME) { $env:HOME } else { $env:USERPROFILE }
    $Config = Join-Path $HomeDir '.config\draupnir\config.yaml'
    $AppDataConfig = Join-Path $env:APPDATA 'draupnir\config.yaml'
    if ((Test-Path $Config) -or (Test-Path $AppDataConfig)) {
        Write-Host 'A draupnir config already exists; left as is.'
    } else {
        $Default = Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'Draupnir'
        $Vault = $env:DRAUPNIR_VAULT_ROOT
        if (-not $Vault) {
            if ([Console]::IsInputRedirected) {
                $Vault = $Default  # no terminal to ask in
            } else {
                $Answer = Read-Host "Vault folder (an existing Obsidian vault works too) [$Default]"
                $Vault = if ($Answer) { $Answer } else { $Default }
            }
        }
        # Forward slashes and single quotes: a double-quoted YAML string would treat `\` as an escape.
        $Vault = $Vault.Trim().Trim('"').Replace('\', '/')
        New-Item -ItemType Directory -Force (Split-Path $Config) | Out-Null
        [IO.File]::WriteAllText($Config, "vault_root: '$($Vault.Replace("'", "''"))'`n")  # UTF-8 without BOM
        Write-Host "Wrote $Config (vault: $Vault)"
    }

    & $Exe skills install
    if ($LASTEXITCODE -ne 0) { Write-Warning 'Some skills were not updated; see the lines above.' }

    Write-Host ''
    Write-Host "Installed $Exe"
    Write-Host 'Open a new terminal, then register a project from inside its repo:'
    Write-Host '  draupnir project add <key> --title "<Title>" --repo <path to the repo>'
}
