# build-flat.ps1
Write-Host "Building Tauri application..." -ForegroundColor Cyan
npm run tauri build

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful, organizing files..." -ForegroundColor Green
    $path = "src-tauri/target/release/bundle"
    
    # Move msi files
    if (Test-Path "$path/msi") {
        Get-ChildItem "$path/msi/*" | Move-Item -Destination $path -Force
        Remove-Item "$path/msi" -Recurse -Force
        Write-Host "Moved MSI files" -ForegroundColor Yellow
    }
    
    # Move nsis files  
    if (Test-Path "$path/nsis") {
        Get-ChildItem "$path/nsis/*" | Move-Item -Destination $path -Force
        Remove-Item "$path/nsis" -Recurse -Force
        Write-Host "Moved NSIS files" -ForegroundColor Yellow
    }
    
    Write-Host "Done! Installers are now in bundle root directory." -ForegroundColor Green
    
    # Show final files
    Write-Host "Final files:" -ForegroundColor Magenta
    Get-ChildItem $path -File | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
} else {
    Write-Host "Build failed!" -ForegroundColor Red
}