# Nginx Config Manager - 完全重启脚本
# 用于清除缓存并重新启动应用

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Nginx Config Manager - 完全重启" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 停止所有相关进程
Write-Host "[1/4] 停止所有相关进程..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object {
    $_.ProcessName -like "*nginx-config-manager*" -or 
    $_.ProcessName -like "*vite*" -or
    $_.ProcessName -like "*node*"
}

if ($processes) {
    Write-Host "找到 $($processes.Count) 个进程，正在停止..." -ForegroundColor Gray
    $processes | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    Write-Host "✓ 进程已停止" -ForegroundColor Green
} else {
    Write-Host "✓ 没有需要停止的进程" -ForegroundColor Green
}
Write-Host ""

# 步骤 2: 清理构建缓存（可选）
Write-Host "[2/4] 清理构建缓存..." -ForegroundColor Yellow
$cleanCache = Read-Host "是否清理 Rust 构建缓存？这会增加启动时间 (y/N)"
if ($cleanCache -eq "y" -or $cleanCache -eq "Y") {
    if (Test-Path "src-tauri/target") {
        Write-Host "正在清理 src-tauri/target..." -ForegroundColor Gray
        Push-Location src-tauri
        cargo clean
        Pop-Location
        Write-Host "✓ 缓存已清理" -ForegroundColor Green
    } else {
        Write-Host "✓ 没有缓存需要清理" -ForegroundColor Green
    }
} else {
    Write-Host "✓ 跳过缓存清理" -ForegroundColor Green
}
Write-Host ""

# 步骤 3: 清理 node_modules/.vite 缓存
Write-Host "[3/4] 清理 Vite 缓存..." -ForegroundColor Yellow
if (Test-Path "node_modules/.vite") {
    Remove-Item -Recurse -Force "node_modules/.vite" -ErrorAction SilentlyContinue
    Write-Host "✓ Vite 缓存已清理" -ForegroundColor Green
} else {
    Write-Host "✓ 没有 Vite 缓存需要清理" -ForegroundColor Green
}
Write-Host ""

# 步骤 4: 启动应用
Write-Host "[4/4] 启动应用..." -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  应用启动后，请按 Ctrl+Shift+R 硬刷新" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 启动应用
npm run tauri dev

