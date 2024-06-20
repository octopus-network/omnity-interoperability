#!ic-repl

load "./process";
identity private "identity.pem";
let file = "perf.md";

//call cid.send_ticket(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_type = variant { Normal }; ticket_time = 1715654809737051178 : nat64; token = "Bitcoin-runes-HOPE•YOU•GET•RICH"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Arbitrum"; action = variant { Transfer }; sender = opt "address_on_Bitcoin"; receiver = "address_on_Arbitrum"; memo = null; });
//output(file, stringify("\n## The cost of send_ticket: ",__cost__, "\n"));

let svg = stringify("The send_ticket Profiling-", $timestamp,".svg");
let send_ticket_cost = flamegraph(cid, "The send_ticket Profiling", svg);
output(file, stringify("\n## The cost of latest update call for send_ticket: ",send_ticket_cost, "\n"));



let process = "process";
let status = call ic.canister_status(record { canister_id = cid });
let current_cycles = status.cycles;
output(process, stringify("let current_timestamp = ",$timestamp,";\n"));
output(process, stringify("let current_cycles = ",current_cycles,";\n"));
let test_cost = sub(init_cycles,current_cycles);
output(process, stringify("let test_cost = ",test_cost,";\n"));
let current_memory_size = status.memory_size;
output(process, stringify("let current_memory_size = ",current_memory_size,";\n"));
let test_memory_increase = sub(current_memory_size,init_memory_size);

output(file, "\n## The omnity_hub status info after send_ticket: \n");
output(file, stringify("> *  canister id: ",cid, "\n"));
//output(file, stringify("> *  wasm size: ",call vp_wasm.size(), "\n"));
output(file, stringify("> *  cid.__get_cycles: ",call cid.__get_cycles(), "\n"));
output(file, stringify("> *  status: ",status, "\n"));
output(file, stringify("> *  canister status: ",status.status, "\n"));
output(file, stringify("> *  memory size: ",status.memory_size, "\n"));
output(file, stringify("> *  memory increase: ",test_memory_increase, "\n"));
output(file, stringify("> *  canister cycles: ",current_cycles, "\n"));
output(file, stringify("> *  settings.freezing_threshold : ",status.settings.freezing_threshold, "\n"));
output(file, stringify("> *  settings.controllers : ",status.settings.controllers, "\n"));
output(file, stringify("> *  settings.memory_allocation : ",status.settings.memory_allocation, "\n"));
output(file, stringify("> *  settings.compute_allocation : ",status.settings.compute_allocation, "\n"));
output(file, stringify("> *  idle_cycles_burned_per_day : ",status.idle_cycles_burned_per_day, "\n"));
output(file, stringify("> *  module_hash : ",status.module_hash, "\n"));

