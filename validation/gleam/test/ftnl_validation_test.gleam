import ftnl_validation
import gleam/dynamic.{type Dynamic}
import gleam/string
import gleeunit
import gleeunit/should

pub fn main() { gleeunit.main() }

fn request_meta(request_id: String, trace_id: String, locale: Dynamic) {
  dynamic.properties([
    #(dynamic.string("requestId"), dynamic.string(request_id)),
    #(dynamic.string("traceId"), dynamic.string(trace_id)),
    #(dynamic.string("locale"), locale),
  ])
}

pub fn request_boundaries_test() {
  request_meta(string.repeat("r", times: 128), string.repeat("t", times: 128), dynamic.string(string.repeat("l", times: 64)))
  |> ftnl_validation.decode_request_meta
  |> should.be_ok

  request_meta(string.repeat("r", times: 129), "trace-1", dynamic.string("en"))
  |> ftnl_validation.decode_request_meta
  |> should.be_error

  request_meta("req-1", "trace-1", dynamic.string("e"))
  |> ftnl_validation.decode_request_meta
  |> should.be_error

  request_meta("req-1", "trace-1", dynamic.nil())
  |> ftnl_validation.decode_request_meta
  |> should.be_error
}

pub fn page_and_problem_boundaries_test() {
  dynamic.properties([#(dynamic.string("limit"), dynamic.int(1))])
  |> ftnl_validation.decode_page_query
  |> should.be_ok

  dynamic.properties([#(dynamic.string("limit"), dynamic.int(101))])
  |> ftnl_validation.decode_page_query
  |> should.be_error

  dynamic.properties([])
  |> ftnl_validation.decode_page_query
  |> should.be_error

  dynamic.properties([
    #(dynamic.string("type"), dynamic.string("urn:test")),
    #(dynamic.string("title"), dynamic.string("bad")),
    #(dynamic.string("status"), dynamic.int(600)),
    #(dynamic.string("requestId"), dynamic.string("req-1")),
  ])
  |> ftnl_validation.decode_problem_details
  |> should.be_error
}
