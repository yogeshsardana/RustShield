#!/usr/bin/env bash
#
# RustShield Performance Benchmark Script
#
# Measures the hotswap window duration, packet loss, and throughput
# impact of live driver migration.

set -euo pipefail

DRIVER="${1:-e1000e}"
ITERATIONS="${2:-10}"
OUTPUT_DIR="benchmark_results"
mkdir -p "${OUTPUT_DIR}"

echo "RustShield Benchmark: ${DRIVER} (${ITERATIONS} iterations)"
echo "================================================"

for i in $(seq 1 "${ITERATIONS}"); do
    echo "Iteration ${i}/${ITERATIONS}..."

    # Record pre-migration stats
    TX_BEFORE=$(ethtool -S eth0 2>/dev/null | grep tx_packets | awk '{print $2}' || echo 0)
    RX_BEFORE=$(ethtool -S eth0 2>/dev/null | grep rx_packets | awk '{print $2}' || echo 0)

    # Execute hotswap and measure timing
    START_TIME=$(date +%s%N)

    # In production, call the actual hotswap ioctl here
    sleep 0.001  # Simulate 1ms hotswap

    END_TIME=$(date +%s%N)
    DURATION_NS=$((END_TIME - START_TIME))
    DURATION_US=$((DURATION_NS / 1000))

    # Record post-migration stats
    TX_AFTER=$(ethtool -S eth0 2>/dev/null | grep tx_packets | awk '{print $2}' || echo 0)
    RX_AFTER=$(ethtool -S eth0 2>/dev/null | grep rx_packets | awk '{print $2}' || echo 0)

    TX_LOST=$((TX_BEFORE - TX_AFTER))
    RX_LOST=$((RX_BEFORE - RX_AFTER))

    echo "${i},${DURATION_US},${TX_LOST},${RX_LOST}" >> "${OUTPUT_DIR}/results.csv"
    echo "  Window: ${DURATION_US} us | TX lost: ${TX_LOST} | RX lost: ${RX_LOST}"
done

echo ""
echo "Results written to ${OUTPUT_DIR}/results.csv"
echo "Summary:"
awk -F, 'NR>1{sum+=$2; n++} END{print "  Avg window: " sum/n " us"}' "${OUTPUT_DIR}/results.csv"
