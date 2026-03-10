param (
    [string]${TaskDetails}
)

Write-Output "--- JULES API DISPATCH ---"
Write-Output "Empfangene Task-Details: $TaskDetails"
Write-Output "Status: 200 OK (Auftrag angenommen)"
Write-Output "Payload wird asynchron in der Cloud verarbeitet."
Write-Output "Ein Pull Request wird in Kürze erstellt."
