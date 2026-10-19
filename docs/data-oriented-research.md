# DINAH — initial AoS/SoA layout evaluation (point 085)

Status: initial exploration documented; **no universal layout is adopted and point 085 is not declared complete**.

## Question and scope

Two representations of the same data are compared (`first = i`, `second = i + 1`):

- **AoS (Array of Structures):** `Vec<DataRow>`, with both fields in each row.
- **SoA (Structure of Arrays):** `SoaBuffer`, with each field in a contiguous column.

The `benches/memory.rs` benchmark measures traversals that read only `first` and traversals that read and combine both fields. The structures are built before measuring, so these results **do not include** construction, conversion, allocation or maintenance cost of either representation. Each case reports the median of seven samples at sizes of 64, 1,024 and 16,384 rows.

## Available reproducible evidence

- Measured code commit: `92069269bbda514702808e769a416dbd519e76d6`.
- Benchmark run: [GitHub Actions run 38206675890](https://github.com/lecodev-26/dinah/actions/runs/38206675890).
- The complete results, environment and manifest were automatically archived in `artifacts/benchmarks/92069269bbda514702808e769a416dbd519e76d6/run-38206675890/`, separated by runner.
- Three independent runs per runner; each case summarizes seven samples per run. The first measured commit `92069269bbda514702808e769a416dbd519e76d6`; the second and third measured the same benchmark code (`114e0f73e05d17bdfd6bb7002a52ee234b72f834` and `8854d3794dfda846f7465920930b6c403a1d3ab8`, respectively). The third run extends the evidence, but three runs are not enough to claim general statistical stability.

## Results at 16,384 rows

Times are nanoseconds per operation as reported by the harness. The percentage expresses the change of SoA relative to AoS: negative favors SoA; positive favors AoS. Values are compared only within each runner.

| Runner | Read one field AoS | Read one field SoA | SoA change | Read both AoS | Read both SoA | SoA change |
|---|---:|---:|---:|---:|---:|---:|
| Linux x86-64 | 0.473 | 0.465 | −1.7 % | 0.886 | 0.895 | +1.0 % |
| Linux ARM64 | 0.469 | 0.407 | −13.2 % | 0.694 | 0.773 | +11.4 % |
| macOS x86-64 | 0.511 | 0.421 | −17.6 % | 0.851 | 0.817 | −4.0 % |
| macOS ARM64 | 0.315 | 0.394 | +25.1 % | 0.684 | 0.732 | +7.0 % |
| Windows x86-64 | 0.378 | 0.513 | +35.7 % | 0.781 | 0.745 | −4.6 % |

## Measurement repetitions

A second complete run was launched across the five runners: [GitHub Actions run 38207649616](https://github.com/lecodev-26/dinah/actions/runs/38207649616). The complete results and environment were archived in `artifacts/benchmarks/114e0f73e05d17bdfd6bb7002a52ee234b72f834/run-38207649616/`. At 16,384 rows, these are the SoA changes relative to AoS in each run (negative favors SoA; positive favors AoS):

| Runner | One field: first run | One field: repetition | Both: first run | Both: repetition |
|---|---:|---:|---:|---:|
| Linux x86-64 | −1.7 % | −1.2 % | +1.0 % | +0.0 % |
| Linux ARM64 | −13.2 % | −2.6 % | +11.4 % | +10.8 % |
| macOS x86-64 | −17.6 % | −1.6 % | −4.0 % | −4.9 % |
| macOS ARM64 | +25.1 % | −9.7 % | +7.0 % | −2.0 % |
| Windows x86-64 | +35.7 % | +33.8 % | −4.6 % | −4.9 % |

The repetition confirms that **there is no universal advantage**, but also that not every large difference is stable: the macOS ARM64 direction changes in both patterns; Linux ARM64 keeps the direction but the one-field advantage shrinks a lot; Windows retains an SoA penalty for reading one field, while for reading both fields it favors SoA by a small margin. On Linux x86-64 the results are practically tied. These results do not justify changing the representation or choosing a default layout.

The intermediate size (1,024 rows) also gives no universal winner: AoS wins the one-field read on all five runners in that run, while the both-fields read varies by runner. At 64 rows, the measurements are too small and quantized to ground a design decision.

A third complete run was completed on five runners: [GitHub Actions run 38208067774](https://github.com/lecodev-26/dinah/actions/runs/38208067774), archived in `artifacts/benchmarks/8854d3794dfda846f7465920930b6c403a1d3ab8/run-38208067774/`. At 16,384 rows, the SoA change relative to AoS was:

| Runner | One field: third run | Both: third run |
|---|---:|---:|
| Linux x86-64 | +8.7 % | +0.3 % |
| Linux ARM64 | −2.9 % | +11.1 % |
| macOS x86-64 | +1.0 % | −3.6 % |
| macOS ARM64 | +23.1 % | +0.5 % |
| Windows x86-64 | +35.6 % | +0.7 % |

The third run reinforces the Windows signal: SoA was roughly 34–36 % slower for the one-field read across the three runs. In contrast, the both-fields differences are usually small, except Linux ARM64, where SoA was around 11 % slower in all three runs. The one-field read changes direction or magnitude on Linux x86-64 and macOS ARM64; macOS x86-64 is nearly tied in the third run. This remains synthetic microbenchmark evidence, not a layout recommendation for real workloads.

## Interpretation and decision

1. SoA can help when the traversal consumes one column and avoids loading fields that are not needed, but the measured effect depends on architecture and size.
2. When both fields are consumed, AoS is competitive and often faster in this sample; SoA must not be presented as a general optimization.
3. Large percentages from a single run do not prove a stable advantage. There are resolution differences, runner noise and compiler/cache effects.
4. The test excludes the cost of creating and maintaining the layout. An architecture change would only be justified for a real workload where the read/write distribution is defined.

**Decision:** keep both types available and do not change the API or the internal representation based on these numbers. Before considering further optimization, repeat the benchmark across several commits/runs per runner, keep the environment configuration, and add a case that measures the construction/conversion cost if a real consumer needs it. Compare medians within each runner and require a repeatable advantage that justifies the complexity.

## Exit criteria for point 085

- [x] Add an AoS/SoA comparative benchmark with equivalent values for reading one field and both fields.
- [x] Archive results and environment from the five runners.
- [x] Document that the initial evidence is mixed and does not authorize a general optimization.
- [x] Complete a third run on the five runners and document which signals repeat and which remain variable.
- [ ] If a representative workload is identified, also measure creation/conversion and the corresponding write pattern.

The three runs do not justify a universal representation or an API change. A representative workload that allows measuring creation/conversion and writes is still pending; for that reason, point 085 remains **in progress**.
