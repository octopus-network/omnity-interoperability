#!ic-repl

load "./prelude.sh";
// identity private "identity.pem";
let file = "perf.md";
let process = "process";

// instrument wasm
let omnity_hub = wasm_profiling("../target/wasm32-unknown-unknown/release/omnity_hub.wasm");
let size = omnity_hub.size();
let init = encode omnity_hub.__init_args(
    variant { Init = record { admin = principal "rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"} }
  );  

// install wasm
let cid = install(omnity_hub, init, null);
// let init_cycles = call cid.__get_cycles();
let status = call ic.canister_status(record { canister_id = cid });
let init_cycles = status.cycles;
let init_memory_size = status.memory_size;
// output canister info
output(process, stringify("#!ic-repl\n\nimport cid = ","\"", cid,"\"",";\n"));
output(process, stringify("let init_cycles = ",init_cycles,";\n"));
output(process, stringify("let init_memory_size = ",init_memory_size,";\n"));

output(file, "\n## The omnity_hub init status info: \n");
output(file, stringify("> *  canister id: ",cid, "\n"));
output(file, stringify("> *  wasm size: ",size, "\n"));
output(file, stringify("> *  cid.__get_cycles: ",call cid.__get_cycles(), "\n"));
//output(file, stringify("> *  status: ",status, "\n"));
output(file, stringify("> *  canister status: ",status.status, "\n"));
output(file, stringify("> *  memory size: ",init_memory_size, "\n"));
output(file, stringify("> *  canister cycles: ",status.cycles, "\n"));
output(file, stringify("> *  settings.freezing_threshold : ",status.settings.freezing_threshold, "\n"));
output(file, stringify("> *  settings.controllers : ",status.settings.controllers, "\n"));
output(file, stringify("> *  settings.memory_allocation : ",status.settings.memory_allocation, "\n"));
output(file, stringify("> *  settings.compute_allocation : ",status.settings.compute_allocation, "\n"));
output(file, stringify("> *  idle_cycles_burned_per_day : ",status.idle_cycles_burned_per_day, "\n"));
output(file, stringify("> *  module_hash : ",status.module_hash, "\n"));

// disable tracing
// call cid.__toggle_tracing();
// let update_prefix = call cid.update_commitment_prefix("ibc");
// output(file, stringify("\n## The cost of update_commitment_prefix : ",__cost_update_prefix, "\n"));

// enable tracing
//call cid.__toggle_tracing();
// disable tracing clear
//call cid.__toggle_entry();