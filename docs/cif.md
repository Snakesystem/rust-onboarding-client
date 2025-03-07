# Cost Insurance and Freight Api Specification

## Get CIF
Endpoint: **GET** `/api/v1/cif/get-data`

Request Header:
- Authorized token (Cookies)

Response Body(200):
```json
{
    "result": true,
    "message": "Get CIF Success",
    "data": {
        "autonid": 1,
        "fullname": "New York",
        "register_date": "London",
        "mobile_phone": "2022-01-01",
        .........
        ........
    }
}
```

Response Body(400):
```json
{
    "result": false,
    "message": "Get CIF Failed",
    "data": "User not found"
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

## Update CIF State 1
Endpoint: **POST** `/api/v1/cif/save-stage-1`

Request Header:
- Authorized token (Cookies)

Request Body:
```json
{
  "stage": 0,
  "autonid": 0,
  "mobile_phone": "string",
  "email": "string",
  "full_name": "string",
  "mother_name": "string",
  "idcard_number": "string",
  "nationality": 0,
  "sex": 0,
  "birth_place": "string",
  "birth_date": "datetime",
  "birth_country": "string",
  "religion": 0,
  "marital_status": 0,
  "education": 0,
  "education_text": "string",
  "copy_id": true,
  "idcard_expire_date": "datetime",
  "idcard_city": 0,
  "idcard_district": "string",
  "idcard_subdistrict": "string",
  "idcard_rt": "string",
  "idcard_rw": "string",
  "idcard_address": "string",
  "idcard_zipcode": "string",
  "domicile_city": 0,
  "domicile_district": "string",
  "domicile_subdistrict": "string",
  "domicile_rt": "string",
  "domicile_rw": "string",
  "domicile_address": "string",
  "domicile_zipcode": "string",
  "residency_status": 0,
  "beneficiary": 0,
  "idcard_file": "string",
  "selfie_file": "string",
  "signature_file": "string"
}
```

Response Body(200):
```json
{
    "result": true,
    "message": "Save CIF Stage 1 Success"
}
```

Response Body(400):
```json
{
    "result": false,
    "message": "Invalid request",
    "data": "ID Card is required, .etc"
}
```

Response Body(500):
```json
{
    "result": false,
    "message": "Internal Server Error",
    "data": "Please check connection server"
}
```