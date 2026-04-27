# Test File Generation

$TestDir = "$env:USERPROFILE\test-peridot-$(Get-Random)"
New-Item -ItemType Directory -Path $TestDir -Force | Out-Null
Set-Location $TestDir

Write-Host "=== PeridotCode File Generation Test ===" -ForegroundColor Cyan
Write-Host "Test directory: $TestDir" -ForegroundColor Yellow
Write-Host ""

# Set logging
$env:RUST_LOG = "info"

Write-Host "Running peridotcode..." -ForegroundColor Green
Write-Host "When the TUI opens, type: Create a platformer game" -ForegroundColor White
Write-Host ""

# Run peridotcode
& "D:\Codingan\antigane\peridotcode\target\release\peridotcode.exe"

Write-Host ""
Write-Host "=== Checking results ===" -ForegroundColor Cyan

$Files = Get-ChildItem -Path $TestDir -File -Recurse

if ($Files.Count -gt 0) {
    Write-Host "✅ SUCCESS! $($Files.Count) files were created:" -ForegroundColor Green
    $Files | Select-Object -First 20 | ForEach-Object { 
        Write-Host "  - $($_.FullName.Replace($TestDir, '.'))" 
    }
} else {
    Write-Host "❌ No files were created" -ForegroundColor Red
    Write-Host "Check the logs above for errors" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Test directory: $TestDir" -ForegroundColor Yellow
Write-Host "You can delete it with: Remove-Item -Recurse -Force '$TestDir'" -ForegroundColor Gray
