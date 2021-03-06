/**
 * Copyright Soramitsu Co., Ltd. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#ifndef TEST_INTEGRATION_EXECUTOR_FIXTURE_PARAM_POSTGRES_HPP
#define TEST_INTEGRATION_EXECUTOR_FIXTURE_PARAM_POSTGRES_HPP

#include "integration/executor/executor_fixture_param.hpp"

namespace iroha {
  namespace integration_framework {
    class TestDbManager;
  }
}  // namespace iroha

namespace executor_testing {

  /**
   * PostgreSQL backend parameter for ExecutorTest.
   * Creates and holds a test database manager object that:
   * - gets PostgreSQL connection options
   * - creates a new working database with a random name
   * - drops the working database when the test suite is complete
   */
  class PostgresExecutorTestParam : public ExecutorTestParam {
   public:
    PostgresExecutorTestParam();

    virtual ~PostgresExecutorTestParam();

    void clearBackendState() override;

    iroha::integration_framework::ExecutorItfTarget getExecutorItfParam()
        const override;

    std::string toString() const override;

   private:
    std::unique_ptr<iroha::integration_framework::TestDbManager> db_manager_;
    iroha::integration_framework::ExecutorItfTarget executor_itf_target_;
  };
}  // namespace executor_testing

#endif /* TEST_INTEGRATION_EXECUTOR_FIXTURE_PARAM_POSTGRES_HPP */
