# Cost Insurance and Freight Api Specification

## Get CIF
Endpoint: **GET** `/api/v1/user/data`

Request Header:
- Authorized token (Cookies)

Response Body(200):
```json
{
    "result": true,
    "message": "Welcome ",
    "data": {
        "autonid": 1,
        "stage": 5,
        "client_id": "",
        "cif_id": "",
        "is_revised": false,
        "is_rejected": false,
        "is_finished": true,
        "account_status": 0,
        "mobile_phone": "6282323443535",
        "email": "example@gmail.com",
        "full_name": "Dani",
        ````````````````
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
Endpoint: **POST** `/api/v1/user/data-pribadi`

Request Header:
- Authorized token (Cookies)

Request Body:
```json
{
    "email": "example@gmail.com",
    "mobile_phone": "6282323443535",
    "full_name": "Dani",
    "mother_name": "Andi",
    "idcard_number": "123456789101112",
    "nationality": 6,
    "sex": 2,
    "birth_place": "JAKARTA",
    "birth_date": "2024-03-03",
    "birth_country": "INDONESIA",
    "religion": 2,
    "marital_status": 1,
    "education": 3,
    "education_text": "",
    "copy_id": false,
    "idcard_expireddate": "2024-03-03",
    "idcard_city": 6,
    "idcard_district": "as",
    "idcard_subdistrict": "sd",
    "idcard_rt": "34",
    "idcard_rw": "23",
    "idcard_address": "43",
    "idcard_zipcode": "43",
    "domicile_city": 9,
    "domicile_district": "hh",
    "domicile_subdistrict": "hh",
    "domicile_rt": "88",
    "domicile_rw": "999",
    "domicile_address": "hh",
    "domicile_zipcode": "9",
    "residence_status": 2,
    "beneficiary_owner": 3,
    "idcard_country": "INDONESIA",
    "idcard_file": "data:image/png;base64,iVBORw0KGgoAAAANSUhEU......",
    "selfie_file": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAPcAAADMCAMAAAC....",
    "signature_file": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAA........"
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

## Update CIF State 2
Endpoint: **POST** `/api/v1/user/data-bank`

Request Header:
- Authorized token (Cookies)

Request Body:
```json
{
    "question_rdn": 6,
    "bank_name": "Bank BCA",
    "bank_account_holder": "Slamet",
    "bank_account_number": "123456789",
    "bank_branch": "Jakarta"
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

## Update CIF State 3
Endpoint: **POST** `/api/v1/user/data-pekerjaan`

Request Header:
- Authorized token (Cookies)

Request Body:
```json
{
    "company_name": "MPC",
    "company_city": 2,
    "company_address": "Jakarta",
    "company_zipcode": "12345",
    "question_npwp": 1,
    "npwp_reason": "df",
    "npwp_number": "123456789",
    "fund_source": 3,
    "fund_source_text": "",
    "occupation": 2,
    "occupation_text": "",
    "nature_bussiness": 4,
    "nature_bussiness_text": "",
    "position": 8,
    "position_text": "",
    "income_peranum": 2,
    "spouse_name": "Andi",
    "spouse_relationship": 4,
    "spouse_occupation": 3,
    "spouse_occupation_text": "",
    "spouse_fund_source": 2,
    "spouse_fund_source_text": "",
    "spouse_position": 2,
    "spouse_income_peranum": 2,
    "spouse_nature_bussiness": 2,
    "spouse_company_name": "MPC",
    "spouse_company_city": 3,
    "spouse_company_address": "Jakarta",
    "spouse_company_zipcode": "1234"
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
    "message": "Validation failed",
    "error": {
        "npwp_reason": "Npwp_reason is required"
    }
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

## Update CIF State 3
Endpoint: **POST** `/api/v1/user/data-pekerjaan`

Request Header:
- Authorized token (Cookies)

Request Body:
```json
{
    "question_1": false,
    "question_1text": "",
    "question_2": false,
    "question_2text": "",
    "question_3": false,
    "question_3text": "",
    "question_4": false,
    "question_4text": "",
    "question_5": false,
    "question_5text": "",
    "question_6": false,
    "question_6text": "",
    "investment_objective": 2,
    "risk": 1,
    "question_fatca": "ss",
    "fatca_1": "ss",
    "fatca_2": "ss",
    "fatca_3": "ss"
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
    "message": "Validation failed",
    "error": {
        "investment_objective": "Investment_objective is required",
        "question_fatca": "Question_fatca is required",
        "fatca_1": "Fatca_1 is required",
        "fatca_3": "Fatca_3 is required",
        "fatca_2": "Fatca_2 is required"
    }
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
