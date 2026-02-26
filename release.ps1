# release.ps1 - 版本号更新 + 构建一体化脚本
# 用法:
#   ./release.ps1          先交互式更新版本号，再构建
#   ./release.ps1 -Bump    只更新版本号
#   ./release.ps1 -Build   只构建（跳过版本号更新）

param(
    [switch]$Bump,
    [switch]$Build
)

$packageJson = "package.json"
$tauriConf   = "src-tauri/tauri.conf.json"
$cargoToml   = "src-tauri/Cargo.toml"
$bundlePath  = "src-tauri/target/release/bundle"

# ---- 版本号更新 ----
function Invoke-BumpVersion {
    $pkg = Get-Content $packageJson -Raw | ConvertFrom-Json
    $current = $pkg.version

    Write-Host ""
    Write-Host "当前版本: " -NoNewline -ForegroundColor Cyan
    Write-Host $current -ForegroundColor Yellow
    Write-Host ""

    $newVersion = Read-Host "请输入新版本号 (直接回车跳过)"

    if ([string]::IsNullOrWhiteSpace($newVersion)) {
        Write-Host "跳过版本号更新。" -ForegroundColor Gray
        return $false
    }

    if ($newVersion -notmatch '^\d+\.\d+\.\d+$') {
        Write-Host "格式错误，版本号必须为 x.y.z 格式（如 0.3.0）" -ForegroundColor Red
        exit 1
    }

    if ($newVersion -eq $current) {
        Write-Host "版本号未变化，跳过更新。" -ForegroundColor Gray
        return $false
    }

    Write-Host ""
    Write-Host "将更新: $current  →  $newVersion" -ForegroundColor Green
    Write-Host ""

    $pkgContent = Get-Content $packageJson -Raw
    $pkgContent = $pkgContent -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$newVersion`""
    Set-Content $packageJson -Value $pkgContent -NoNewline
    Write-Host "  ✓ package.json" -ForegroundColor Green

    $tauriContent = Get-Content $tauriConf -Raw
    $tauriContent = $tauriContent -replace '"version"\s*:\s*"[^"]*"', "`"version`": `"$newVersion`""
    Set-Content $tauriConf -Value $tauriContent -NoNewline
    Write-Host "  ✓ src-tauri/tauri.conf.json" -ForegroundColor Green

    $cargoContent = Get-Content $cargoToml -Raw
    $cargoContent = $cargoContent -replace '(?m)^(version\s*=\s*)"[^"]*"', "`${1}`"$newVersion`""
    Set-Content $cargoToml -Value $cargoContent -NoNewline
    Write-Host "  ✓ src-tauri/Cargo.toml" -ForegroundColor Green

    Write-Host ""
    Write-Host "版本号已同步更新为 $newVersion" -ForegroundColor Cyan
    return $true
}

# ---- 构建 + 整理产物 ----
function Invoke-Build {
    Write-Host ""
    Write-Host "开始构建..." -ForegroundColor Cyan
    npm run tauri build

    if ($LASTEXITCODE -ne 0) {
        Write-Host "构建失败！" -ForegroundColor Red
        exit 1
    }

    Write-Host "构建成功，整理产物..." -ForegroundColor Green

    foreach ($sub in @("msi", "nsis")) {
        $subPath = "$bundlePath/$sub"
        if (Test-Path $subPath) {
            Get-ChildItem "$subPath/*" | Move-Item -Destination $bundlePath -Force
            Remove-Item $subPath -Recurse -Force
            Write-Host "  ✓ 移动 $sub 文件" -ForegroundColor Yellow
        }
    }

    Write-Host ""
    Write-Host "完成！安装包位于 $bundlePath" -ForegroundColor Green
    Write-Host ""
    Get-ChildItem $bundlePath -File | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
}

# ---- 入口 ----
if ($Bump -and -not $Build) {
    Invoke-BumpVersion | Out-Null
} elseif ($Build -and -not $Bump) {
    Invoke-Build
} else {
    # 默认：先 bump 再 build
    Invoke-BumpVersion | Out-Null
    Invoke-Build
}
