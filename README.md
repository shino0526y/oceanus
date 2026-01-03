# Oceanus

Rustで実装されたPACSです。

## DICOM Server

DICOMサーバー。

### 対応するサービス

- Verification
  - Verification SOP Class (1.2.840.10008.1.1)
- Storage
  - Computed Radiography Image Storage (1.2.840.10008.5.1.4.1.1.1)
  - Digital X-Ray Image Storage - For Presentation (1.2.840.10008.5.1.4.1.1.1.1)
  - Digital Mammography X-Ray Image Storage - For Presentation (1.2.840.10008.5.1.4.1.1.1.2)
  - CT Image Storage (1.2.840.10008.5.1.4.1.1.2)
  - MR Image Storage (1.2.840.10008.5.1.4.1.1.4.2)
  - Secondary Capture Image Storage (1.2.840.10008.5.1.4.1.1.7)
  - X-Ray Angiographic Image Storage (1.2.840.10008.5.1.4.1.1.12.1)
  - X-Ray Radiofluoroscopic Image Storage (1.2.840.10008.5.1.4.1.1.12.2)

### 対応する転送構文

- Implicit VR Little Endian: Default Transfer Syntax for DICOM (1.2.840.10008.1.2)
- Explicit VR Little Endian (1.2.840.10008.1.2.1)

### 対応する文字セット

以下の`Specific Character Set`の値に対応。

- `""` (デフォルト文字セット)
- `"ISO_IR 13"`
- `"ISO_IR 192"`
- `"ISO 2022 IR 6\ISO 2022 IR 87"`
- `"ISO 2022 IR 13\ISO 2022 IR 87"`
- `"ISO 2022 IR 6\ISO 2022 IR 13\ISO 2022 IR 87"`
