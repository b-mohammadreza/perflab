// matmul.cpp defaults (v0)
// MAT_DIM_N   = 256         // 256 x 256
// ITERATIONS  = 8
// Input pattern:
//    A[i][j] = (i + j)         & 255
//    B[i][j] = (i * 3 + j * 7) & 255
// Verification:
//    C[0][0] = 4624000
//    C[0][1] = 4570880
//    C[1][0] = 4591872
//    C[1][1] = 4602496

#include <cstdint>
#include <iostream>
#include <vector>
#include <chrono>

int main([[maybe_unused]] int argc,[[maybe_unused]] char* argv[])
{
    constexpr uint32_t expected_c00 = 4624000;
    constexpr uint32_t expected_c01 = 4570880;
    constexpr uint32_t expected_c10 = 4591872;
    constexpr uint32_t expected_c11 = 4602496;

    constexpr uint32_t iter_num     = 8;
    constexpr uint32_t mat_size     = 256;


    // Init matrices with all 0
    auto start_tick = std::chrono::steady_clock::now();
    std::vector<std::vector<uint32_t>> mat_a(256, std::vector<uint32_t>(256, 0));
    std::vector<std::vector<uint32_t>> mat_b(256, std::vector<uint32_t>(256, 0));
    std::vector<std::vector<uint32_t>> mat_c(256, std::vector<uint32_t>(256, 0));   // C =A x B

    // Init A and B using the input pattern
    for (uint32_t row = 0; row < mat_size; ++row)
    {
        for (uint32_t col = 0; col < mat_size; ++col)
        {
            mat_a[row][col] = (row + col)         & 0xFF;
            mat_b[row][col] = (row * 3 + col * 7) & 0xFF;
        }
    }

    auto reinit_mat_c = [&mat_c]() {
        for (uint32_t row = 0; row < mat_size; ++row)
        {
            for (uint32_t col = 0; col < mat_size; ++col)
            {
                mat_c[row][col] = 0;
            }
        }
    };
    auto end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds init_diff_ns = end_tick - start_tick;

    // Hot Loop
    start_tick = std::chrono::steady_clock::now();
    for (uint32_t it = 0; it < iter_num; ++it)
    {
        if (it > 0)
            reinit_mat_c();

        for (uint32_t row = 0; row < mat_size; ++row)
        {
            for (uint32_t col = 0; col < mat_size; ++col)
            {
                for (uint32_t aux = 0; aux < mat_size; ++aux)
                {
                    // TODO: Optimize
                    // for performance/sanity: The loop order matters a lot for cache 
                    // (because B is accessed “by column”)
                    mat_c[row][col] += mat_a[row][aux] * mat_b[aux][col];
                }
            }
        }
    }
    end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds compute_diff_ns = end_tick - start_tick;

    // Teardown
    start_tick = std::chrono::steady_clock::now();
    [[maybe_unused]] volatile uint32_t result_c00 = mat_c[0][0];
    end_tick = std::chrono::steady_clock::now();
    std::chrono::nanoseconds teardown_diff_ns = end_tick - start_tick;

    // Verification:
    if (mat_c[0][0] != expected_c00 ||
        mat_c[0][1] != expected_c01 ||
        mat_c[1][0] != expected_c10 ||
        mat_c[1][1] != expected_c11)
    {
        std::cerr   << "matmul verification failed. "
                    << "Expected C[0][0]: "
                    <<  expected_c00
                    << " - Calculated C[0][0]: "
                    << mat_c[0][0]

                    << " - Expected C[0][1]: "
                    <<  expected_c01
                    << " - Calculated C[0][1]: "
                    << mat_c[0][1]

                    << " - Expected C[1][0]: "
                    <<  expected_c10
                    << " - Calculated C[1][0]: "
                    << mat_c[1][0]

                    << " - Expected C[1][1]: "
                    <<  expected_c11
                    << " - Calculated C[1][1]: "
                    << mat_c[1][1]

                    << std::endl;

        return 1;
    }
    
    std::cout   << "{\"bench\":\"matmul\","
                << "\"params\":{\"n\":" << mat_size << ",\"iters\":" << iter_num <<"},"
                << "\"phases_ns\":{\"init\":" << init_diff_ns.count() 
                << ",\"compute\":" << compute_diff_ns.count() 
                << ",\"teardown\":" << teardown_diff_ns.count() << "},"
                << "\"check\":{\"c00\":" << expected_c00 << ",\"c01\":" << expected_c01 << ",\"c10\":" << expected_c10 << ",\"c11\":" << expected_c11 << "}"
                << "}"
                << std::endl;

    return 0;
}