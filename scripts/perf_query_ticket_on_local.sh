#!ic-repl

load "./process";
identity private "identity.pem";
let file = "perf.md";

call cid.query_tickets(opt "Bitcoin" ,0 ,12);
output(file, stringify("\n## The cost of query_ticket: ",__cost__, "\n"));
let svg = stringify("The query_ticket Profiling-", $timestamp,".svg");
let query_ticket_cost = flamegraph(cid, "The query_ticket Profiling", svg);
output(file, stringify("\n## The cost of latest update call for query_ticket: ",query_ticket_cost, "\n"));



// enable tracing clear 
//call cid.__toggle_entry();
// disable tracing
//call cid.__toggle_tracing();
// enable tracing
//call cid.__toggle_tracing();
// disable tracing clear
//call cid.__toggle_entry();



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

output(file, "\n## The omnity_hub status info after query_ticket: \n");
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

