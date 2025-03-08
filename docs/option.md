# Option List Api Specification

## Get CIF
Endpoint: **GET** `/api/v1/option/nationality`

Request Header:
- Authorized token (Cookies)

Response Body(200):
```json
{
    "result": true,
    "message": "Get CIF Success",
    "data": [
        {
            "data_id": 1,
            "code": "AX",
            "description": "AALAND ISLANDS"
        },
        {
            "data_id": 1,
            "code": "ID",
            "description": "Indonesian"
        },
    ]
}
```

Response Body(400):
```json
{
    "result": false,
    "message": "Data not found",
    "data": [ ]
}
```

Response Body(500):
```json
{
    "result": false,
    "message": "Get CIF Failed",
    "data": "Internal Server Error"
}
```