/*
 * Copyright (C) 2022 Vaticle
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

package com.vaticle.typedb.client.api.logic;

import com.vaticle.typedb.client.api.TypeDBTransaction;
import com.vaticle.typeql.lang.pattern.Pattern;

import javax.annotation.CheckReturnValue;

public interface Rule {

    @CheckReturnValue
    String getLabel();

    @CheckReturnValue
    Pattern getWhen();

    @CheckReturnValue
    Pattern getThen();

    @CheckReturnValue
    Rule.Remote asRemote(TypeDBTransaction transaction);

    @CheckReturnValue
    boolean isRemote();

    interface Remote extends Rule {

        void setLabel(String label);

        void delete();

        @CheckReturnValue
        boolean isDeleted();
    }
}