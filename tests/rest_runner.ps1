function Test-Aegis {
    param (
        [string]$Uri = "http://127.0.0.1:8080",
        [int]$TimeoutSec = 5
    )    
    $Headers = @{
        "X-Client" = "Aegis-Tester"
        "Content-Type" = "application/json"
        "X-Token" = "qmwnebrvtcyxuzilokpjahsgdf1620374958"
    }
    
    $Body = @{ 
        "id" = "1"
        "name" = "LifeTester"
        "permissions" = "admin"
        "is_active" = "partial"
        "message" = "Hi from PowerShell" 
    } | ConvertTo-Json

    try {
        # Делаем запрос
        $response = Invoke-RestMethod -Uri $Uri `
                                     -Method Post `
                                     -Headers $Headers `
                                     -Body $Body `
                                     -TimeoutSec $TimeoutSec `
                                     -ResponseHeadersVariable responseHeaders

        Write-Host "`n=== RESPONSE BODY ===" -ForegroundColor Cyan
        $response | Format-List | Out-String

        Write-Host "=== RESPONSE HEADERS ===" -ForegroundColor Yellow
        $responseHeaders | Format-Table
    }
    catch {
        $errorBody = $_.Exception.Response.Content.ReadAsStringAsync().Result
        Write-Host "Ошибка сервера: $errorBody" -ForegroundColor Red
    }
}

Test-Aegis -Uri "http://127.0.0.1:8080" -TimeoutSec 5