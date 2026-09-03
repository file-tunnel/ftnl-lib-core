import ftnl_validation_server
import gleam/dynamic.{type Dynamic}
import gleeunit
import gleeunit/should

pub fn main() { gleeunit.main() }

fn repeat_dynamic(value: Dynamic, times: Int) -> List(Dynamic) {
  case times {
    0 -> []
    _ -> [value, ..repeat_dynamic(value, times - 1)]
  }
}

fn actor(user_id: String, roles: List(Dynamic)) {
  dynamic.properties([
    #(dynamic.string("userId"), dynamic.string(user_id)),
    #(dynamic.string("roles"), dynamic.list(roles)),
  ])
}

fn context(source_ip: Dynamic) {
  dynamic.properties([
    #(dynamic.string("public"), dynamic.properties([
      #(dynamic.string("requestId"), dynamic.string("req-1")),
      #(dynamic.string("traceId"), dynamic.string("trace-1")),
    ])),
    #(dynamic.string("actor"), actor("user-1", [dynamic.string("tunnel-writer")])),
    #(dynamic.string("sourceIp"), source_ip),
  ])
}

pub fn actor_boundaries_test() {
  actor("", [])
  |> ftnl_validation_server.decode_trusted_actor
  |> should.be_error

  actor("user-1", repeat_dynamic(dynamic.string("tunnel-writer"), 65))
  |> ftnl_validation_server.decode_trusted_actor
  |> should.be_error
}

pub fn context_and_command_boundaries_test() {
  context(dynamic.string("not-an-ip"))
  |> ftnl_validation_server.decode_server_request_context
  |> should.be_error

  context(dynamic.nil())
  |> ftnl_validation_server.decode_server_request_context
  |> should.be_error

  dynamic.properties([
    #(dynamic.string("operationId"), dynamic.string("tunnels.create")),
    #(dynamic.string("context"), context(dynamic.string("127.0.0.1"))),
    #(dynamic.string("payload"), dynamic.properties([])),
  ])
  |> ftnl_validation_server.decode_internal_command
  |> should.be_ok

  dynamic.properties([
    #(dynamic.string("operationId"), dynamic.string("")),
    #(dynamic.string("context"), context(dynamic.string("127.0.0.1"))),
    #(dynamic.string("payload"), dynamic.properties([])),
  ])
  |> ftnl_validation_server.decode_internal_command
  |> should.be_error
}
