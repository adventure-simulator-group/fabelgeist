Following the latest ownership clarification, #826 will preserve the existing
checked launch/byte owners and concrete native error causes. Bruno has since
explicitly assigned this agent ownership of the shared core GPU functions after
#774's merge. The #780 agent and I agreed the exact receiving subset:

- `ComputePipeline` construction, preparation, compilation, interface/cache
  validation; `ComputeShader::new`, `ReflectedInterface::new`, `shader::parse_naga`.
- `ComputePass::record` and its dispatch adapter.
- `FastPath::new`/`prepare`, uniform size/stride, `CompiledLayout::new`,
  `UniformArena::allocate`, prepared dynamic offsets and native dispatch.
- `KernelBatch::dispatch`/`copy_buffer` native receiving contracts.

#780 retains its producer, count/layout and device/source cache families. We
will exchange one exact source lineage before integration. This assignment does
not extend to the retained hair, fitting, body, armor, metal, engraving or other
whole-function claims enumerated above. Two local hair adapters inside those
claims are being restored and excluded from the production handoff.

Please acknowledge the later core assignment in the handover, and identify who
will implement the remaining claimed connected families after the initial
audits, including the receiving signatures and exact integration revision when
available. Those paths remain required dependencies: no source completion or
final clearance is claimed, and #826 has not pushed this checkpoint or requested
hosted CI.
