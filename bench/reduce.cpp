// reduce.cpp defaults (v0)
// INPUT_SIZE_N                = 33554432     // 32 * 1024 * 1024 elements
// ITERATIONS                  = 150
// Input Pattern: a[i]         = (i % 1024)
// Expected verification value = 17163091968  // N / 1024 = 32,768 _____ 0 + 1 + … + 1023 = 1023*1024/2 = 523,776 _____ 523,776 * 32,768 = 17,163,091,968 

#include <cstdint>
#include <iostream>
#include <vector>
#include <chrono>

int main([[maybe_unused]] int argc,[[maybe_unused]] char* argv[])
{
    constexpr uint64_t expected_reduce = 17163091968;

    constexpr uint32_t iter_num        = 150;
    constexpr uint64_t input_size      = ((32 << 10) << 10);

    // We need to create an array
    /* If we store inputs in an array and then sum them, we’re benchmarking a realistic mix of:
    *   memory reads (cache/memory bandwidth)
    *   loop optimizations (vectorization/unrolling)
    */
    auto start_tick = std::chrono::steady_clock::now();
    std::vector<uint32_t> input_arr;

    // Init the array using the input pattern
    for (uint32_t elem = 0; elem < input_size; ++elem)
    {
        input_arr.push_back(elem & 0x3FF);
    }
    auto end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds init_diff_ns = end_tick - start_tick;

    // Hot loop
    start_tick = std::chrono::steady_clock::now();
    uint64_t reduce = 0;

    for (uint32_t it = 0; it < iter_num; ++it)
    {
        reduce = 0;
        for (uint32_t elem = 0; elem < input_size; ++elem)
        {
            reduce += input_arr[elem];
        }
    }
    end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds compute_diff_ns = end_tick - start_tick;

    // Teardown
    start_tick = std::chrono::steady_clock::now();
    [[maybe_unused]] volatile uint64_t result_reduce = reduce;
    end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds teardown_diff_ns = end_tick - start_tick; 

    // Verification - now only on the last reduce
    if (expected_reduce != reduce)
    {
        std::cerr   << "reduce verification failed. Expected: " 
                    << expected_reduce 
                    << " - Calculated: " 
                    << reduce 
                    << std::endl; 
        return 1;
    }

    std::cout   << "{\"bench\":\"reduce\","
                << "\"params\":{\"n\":" << input_size << ",\"iters\":" << iter_num << "},"
                << "\"phases_ns\":{\"init\":" << init_diff_ns.count() 
                << ",\"compute\":" << compute_diff_ns.count() 
                << ",\"teardown\":" << teardown_diff_ns.count() << "},"
                << "\"check\":{\"sum\":" << expected_reduce << "}"
                << "}"
                << std::endl;

    return 0;
}
