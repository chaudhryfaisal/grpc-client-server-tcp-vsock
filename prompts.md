# Initial Promp 
create complete PRD for ai coding agent, keep tasks small and start from skeleton of the project and build up the functionality
agent should generate tasks list and save them in tasks.md and check off tasks as they are completed

generate prd for maximize performance and minimal latency grpc client and server in rust which supports both TCP and VSOCK
- maximize performance over single or multiple grpc connections, for single or multiple threads
- grpc should have two services, echo service which will log and echo back payload and crypto service to sign RSA and ECC using rust ring create over grpc
- client should choose which key type and algorithm to use, only support one algorithm per key type
- Server must generate both keys at startup
- use logging crate and have detailed logging at configurable logging level
- client or server should never panic or unwrap and always return error to caller
- generate single function test to test end to end functionality to launch the server, then client to connect and perform echo, and crypto ecc and rsa sign operations.
- add benchmark bin which should have configurable connections and threads to measure performance over single threads or multiple thread, at configureable request per second
- generate all code in single crate with client, server and benchmark binaries
- dont need cicd or documentation in initial phase
- focus on end to end working example before writing test cases and benchmark client
- generate minimal makefile to build, test, run client, run server, and benchmark

# Kilo Code Orchestrator Prompt 
continue to implement prd from @/prd.md , generate tasks list and save them in tasks.md and check off tasks as they are completed, if tasks.md already exist then resume work to complete remaining tasks.md

# Kilo Code Memory Bank Setup
```bash
# https://kilocode.ai/docs/advanced-usage/memory-bank

mkdir -p .kilocode/rules/memory-bank/
cp prd.md .kilocode/rules/memory-bank/brief.md # edit manually
curl -Lo .kilocode/rules/memory-bank-instructions.md https://kilocode.ai/docs/downloads/memory-bank.md
# Switch to Architect mode
# Check if a best available AI model is selected
# Ask Kilo Code to "initialize memory bank"
# Wait for Kilo Code to analyze your project and initialize the Memory Bank files
# Verify the content of the files to see if the project is described correctly. Update the files if necessary.

```