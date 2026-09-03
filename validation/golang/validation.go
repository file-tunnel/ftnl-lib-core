package validation
import("bytes";"encoding/json";"fmt";"io";"github.com/go-playground/validator/v10")
const ContractVersion="ores.validation.v1"
var validate=validator.New(validator.WithRequiredStructEnabled())
type RequestMeta struct{RequestID string `json:"requestId" validate:"required,min=1,max=128"`;TraceID string `json:"traceId" validate:"required,min=1,max=128"`;Locale string `json:"locale,omitempty" validate:"omitempty,min=2,max=64"`}
type PageQuery struct{Limit uint16 `json:"limit" validate:"gte=1,lte=100"`;Cursor string `json:"cursor,omitempty" validate:"omitempty,min=1,max=512"`}
type ProblemDetails struct{Type string `json:"type" validate:"required,min=1,max=512"`;Title string `json:"title" validate:"required,min=1,max=256"`;Status uint16 `json:"status" validate:"gte=400,lte=599"`;Detail string `json:"detail,omitempty" validate:"omitempty,max=4096"`;RequestID string `json:"requestId" validate:"required,min=1,max=128"`}
func Validate(v any)error{if err:=validate.Struct(v);err!=nil{return fmt.Errorf("public validation failed: %w",err)};return nil}
func DecodeAndValidate[T any](data []byte)(T,error){var v T;d:=json.NewDecoder(bytes.NewReader(data));d.DisallowUnknownFields();if err:=d.Decode(&v);err!=nil{return v,fmt.Errorf("decode public value: %w",err)};if err:=d.Decode(&struct{}{});err!=io.EOF{if err==nil{return v,fmt.Errorf("decode public value: trailing JSON value")};return v,fmt.Errorf("decode public value: %w",err)};if err:=Validate(v);err!=nil{return v,err};return v,nil}
