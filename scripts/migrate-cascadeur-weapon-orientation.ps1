param(
    [ValidateSet('migrate', 'verify', 'all')]
    [string]$Mode = 'all'
)

# Usage: powershell -File scripts/migrate-cascadeur-weapon-orientation.ps1 -Mode all
# The wrapper temporarily registers its CLI modules with Cascadeur and restores
# the user's settings even when a scene fails.

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$sourceRoot = Join-Path $repo 'assets_src\biped'
$workRoot = Join-Path $repo 'target\cascadeur_weapon_orientation_migration'
$cascadeur = 'C:\Program Files\Cascadeur\cascadeur.exe'
$settingsPath = Join-Path $env:LOCALAPPDATA 'Nekki Limited\Cascadeur\settings.json'
$settingsBackup = Join-Path $workRoot 'settings.json.backup'
$cliModules = Join-Path $workRoot 'cli_modules'
$files = Get-ChildItem -LiteralPath $sourceRoot -Recurse -Filter '*.casc' |
    Where-Object { $_.Name -notlike '*.orientation-migration.tmp.casc' } |
    Sort-Object FullName

function Invoke-CascadeurPass {
    param(
        [string]$Module,
        [string]$StatusFolder,
        [string[]]$AcceptedStatuses
    )

    $index = 0
    foreach ($file in $files) {
        $index += 1
        $relative = [IO.Path]::GetRelativePath($sourceRoot, $file.FullName)
        $statusPath = Join-Path (Join-Path $workRoot $StatusFolder) ([IO.Path]::ChangeExtension($relative, '.txt'))
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $statusPath) | Out-Null
        Remove-Item -LiteralPath $statusPath -ErrorAction SilentlyContinue
        $env:CASCADEUR_MIGRATION_FILE = $relative
        $process = Start-Process -FilePath $cascadeur -ArgumentList '--single-user-mode', '--run-script', $Module -WindowStyle Hidden -PassThru
        $deadline = [DateTime]::UtcNow.AddMinutes(3)
        while (-not (Test-Path -LiteralPath $statusPath)) {
            if ([DateTime]::UtcNow -ge $deadline) {
                throw "Cascadeur timed out for $relative ($Module)"
            }
            Start-Sleep -Seconds 2
        }
        $status = (Get-Content -Raw -LiteralPath $statusPath).Trim()
        if ($AcceptedStatuses -notcontains $status) {
            throw "Cascadeur returned '$status' for $relative ($Module)"
        }
        Write-Output "[$index/$($files.Count)] $status $relative"
        if (-not $process.WaitForExit(30000)) {
            throw "Cascadeur did not exit after processing $relative ($Module)"
        }
    }
    Remove-Item Env:CASCADEUR_MIGRATION_FILE -ErrorAction SilentlyContinue
}

if (Get-Process -Name cascadeur -ErrorAction SilentlyContinue) {
    throw 'Close Cascadeur before running this migration.'
}

New-Item -ItemType Directory -Force -Path $workRoot, $cliModules | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'migrate_cascadeur_weapon_orientation.py') -Destination $cliModules -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify_cascadeur_weapon_orientation.py') -Destination $cliModules -Force
Copy-Item -LiteralPath $settingsPath -Destination $settingsBackup -Force

$settings = Get-Content -Raw -LiteralPath $settingsPath | ConvertFrom-Json
$cliPath = $cliModules.Replace('\', '/')
$settings.Python.Path = @(@($settings.Python.Path) + $cliPath | Select-Object -Unique)
$settings | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $settingsPath -Encoding utf8
$env:FABELGEIST_ROOT = $repo

try {
    if ($Mode -in @('migrate', 'all')) {
        Invoke-CascadeurPass `
            -Module 'migrate_cascadeur_weapon_orientation' `
            -StatusFolder 'migration_status' `
            -AcceptedStatuses @('migrated', 'already-current', 'skipped-no-weapon-pair')
    }

    if ($Mode -in @('verify', 'all')) {
        Invoke-CascadeurPass `
            -Module 'verify_cascadeur_weapon_orientation' `
            -StatusFolder 'verification_status' `
            -AcceptedStatuses @('verified', 'skipped-no-weapon-pair')
    }
}
finally {
    Remove-Item Env:CASCADEUR_MIGRATION_FILE -ErrorAction SilentlyContinue
    Remove-Item Env:FABELGEIST_ROOT -ErrorAction SilentlyContinue
    Copy-Item -LiteralPath $settingsBackup -Destination $settingsPath -Force
}
