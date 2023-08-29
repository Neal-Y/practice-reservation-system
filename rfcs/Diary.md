# Project Diary

- Feature Name: diary record
- Start Date: 2023-07-27 10:52:11

## Summary

To document the novel technologies and concepts that I am mastering.

## Motivation

to adopt the professional habit of an engineer using English for documentation, I utilize this approach. It not only enhances my ability to articulate problems and express my thoughts, but it also cultivates my skills in pinpointing key points and communicating fluently.

## 07/26

### Title: Intro With Pre-work

it's start from I want to practice the reservation system and it can be used by a number of type of booking systems, Today I learned how to pre-prepared settings, for example, I could have .**github/workflow/build.yml** that automatically triggers the Continuous Integration (CI) process whenever I push or pull code. Also learned how to set up the pre-commit, it's a great way to run a bunch of commands when u type "git commit -a", that means I can write some scripts like **cargo check**, **cargo deny**, **cargo nextest(further test)** and run all of these at the same time.

## 07/27

## Title: Core Logic

Today, I am in the process of developing a concurrent reservation system, where a primary challenge is conflict avoidance. Initially, I considered resolving this issue at the application level by comparing the time slots in the database with those selected by the user to ensure availability. However, this approach has a significant drawback: if User A is contemplating a reservation for a particular time slot, User B might secure that slot first, preventing A from booking.

In this situation, even if User A could initially book a time slot, a conflict arises because User B submitted the request sooner. Regardless of how I tackle this problem at the application level, complex locking issues seem inevitable, leading to considerable overhead.

Consequently, I am contemplating utilizing PostgreSQL's EXCLUDE constraints as a solution, applying restrictions directly at the database core level. This way, I can ensure data consistency at the database level and bypass the complexity at the application level. This method incurs less overhead and effectively avoids concurrent conflicts.

```sql
SELECT int4range(10,20) && int4range(1, 9);
```
**that return false, cuz that isn't overlapping**

## 07/28

### Title: Database Schema Intro

introducing the database "schema" is kind of like C/C++ "namespace" which means that can let all of set up packaged as a one object, make all items better manageable.

why I need to use "gist search" ? The main reason for this choice is that it allows the 'resource_id' to be compared while also executing commands to determine overlap. In other words, it gives me the capability to use **more than one command at the same time**, which is why I chose to use the GiST (Generalized Search Tree) approach. also we created a "reservation_changes" table which means when we have any changes and the trigger will save the change's details insert into the database.

Finally, we implemented a trigger function in PostgreSQL, tasked with logging CRUD operations to the 'reservation_changes' table. However, we must remain mindful of performance implications. High concurrency, brought on by a multitude of simultaneous connections, requires us to ensure that system stability isn't compromised by this logging process."

## 07/29

### Title: Start To Code

It's been a long time, finally I code some smell things that use tonic and gRPC, I use tonic-build crate in the build.rs file which can build the reservation.proto and then BOOM we have reservation.rs in the pb dir that is cuz it can converted protobuf into rs, it's same concept as "SCHEMA", also I learned that "tokei" which is a counting line of code application, that's it, it's wonderful day again. by the way, I'm traveling so maybe not to detail but I do my best : ）.

## 07/31

### Title: use sqlx build migration(遷移)

First, it's important to understand that 'sqlx' is an ORM (Object Relational Mapping) system crate, similar to SeaORM, but with a deeper integration with the database. Once familiar with this concept, I moved on to learning about database migrations. Migrations are used when changes to the database's schema are needed. I generally use the command 'sqlx migrate add 'filename' -r' for this purpose, where '-r' stands for revision migration. Each migration created has a unique timestamp. Every migration is composed of two files, 'up' and 'down'. The 'up' file executes the commands necessary to implement the migration, while the 'down' file allows for the migration to be rolled back if needed.

## 08/01

### Title: build the functions about reservation(1)

learned how to build the trait about what function I need to implement. some basic function like reserve/delete/update etc. to be continued...

## 08/02

### Title: build the functions about reservation(2)

While planning the necessary functions for ReservationManager, I encountered various issues such as a lack of time scope and time conversion (from timestamp to chrono::Utc), but I'm persisting and making progress. by the way, today's **BIGGEST** part is HOW can I deal with the database "status"(postgres) type can not convert to ReservationStatus.**(finished)**

## 08/03

### Title: build the functions about reservation(3)

Today I add new notes to documenting some error when I make this project also changed some README.md, and next time I will be implementing the remaining functions.

- [x] **TODO: I need to fix my git... .pre-commit-config.yaml**

## 08/04

### Title: pre-commit problem

I need to check my configuration to see where the problem is, but I don't know how to do it. I'll try to figure it out a several days, before I solve this problem I should to move on to next step, to implement remaining functions.

## 08/06

### Title: due to unittest so build mock database to do some connection

First and foremost, I've adopted the good practice of using Test-Driven Development (TDD) to ensure my code during refactoring, with the aim of avoiding potential problems that may arise when communicating with external systems. In this process, I've encountered two main challenges:

Testing the Database Interaction: Previously, when testing the network layer in unit tests, we could mock requests and self-answer. However, now we need to interact with the database, requiring a more authentic data source. To resolve this, I've created a mock database to perform a series of migrations during testing. This approach allows us to operate on a relatively real database without interfering with the actual one, then delete the mock database after satisfying the unit test.

Clear Conflict Messages with sqlx::PgDatabase::Error: While further testing for conflict reserves, I needed to understand the problem through error messages. However, PgDatabaseError didn't provide a way to return a very "clear" conflict problem to the user. Since PgDatabaseError doesn't offer a better method for detailed error information, we had to implement our own function to parse a large amount of raw data obtained from get_raw() within PgDatabaseError.

- [x] TODO: write a parser

## 08/07

### Title: fixing cargo script dut to target conflict problem

The issue I encountered is that when using four cargo scripts, they conflict with each other due to the previously compiled files. This leads to the 'file were modified by this hook' error, as the next test requires recompilation to meet the script's criteria. Currently, I'm storing their respective compiled files separately, but I feel this isn't an ideal solution, and I will continue to look for a better way.

Ps: If I fixed I'll document it to [Error note](./error-notes.md)

## 08/08

### Title: write a parser to catch errors and describe them as detailed as possible

1. Printing Details of a Specific Error: To print the details of a particular error, the ReservationConflictInfo enum is used, encompassing two cases: successful (Parsed) and unsuccessful (Unparsed).
2. Structure for Successful Printing: Successful printing utilizes the ReservationConflict structure, containing two separate structures to represent two kinds of Reserve scenarios, encapsulated by ReservationWindow.
3. Newly Added Parsing Structure: A new structure, ParsedInfo, was added to hold parsed paired HashMap data. Regular expressions, .try_fold(), and .and_then() methods were applied for parsing.
4. Error Handling Approach: Reflected on the solutions from a previous todo project, gaining insights into managing errors in the current project.

- [x] **TODO:familiar my function**
- [x] **TODO:write some error handling**

## 08/09

### Title: wrote a parser to catch errors and describe them as detailed as possible

While developing conflict tests for the reservation system, I created a function named reserve_conflict_should_reject() to handle booking conflicts specifically. When a conflicting reservation occurs, I expected an error to be triggered in the database, and I stored it in the err variable.

I began by defining a ConflictReservation error and placing it into ReservationConflictInfo, then manually implemented the From trait for it. During the parsing of this error, I utilized the FromStr trait and regular expressions to interpret the error message into the ReservationConflict structure.

Since this structure involves multiple parts like ReservationWindow, I implemented the FromStr and TryFrom traits for these parts individually. In dealing with dates, I paid special attention to time zones, using a specific format.

Through these series of actions, I was able to effectively manage reservation conflicts and ensure the accuracy of the tests.

## 08/10

### Title: finished the reservation related function

1. 查詢語句與類型問題
我開始遇到了資料庫查詢與ID類型不匹配的問題。在查詢語句中，我嘗試使用字串類型的ID，但資料庫中卻是UUID類型。這引發了一個錯誤，具體表現為PostgreSQL的“operator does not exist: uuid = text”錯誤。為了解這個問題，我必須將ID從字串轉換為UUID，並在查詢語句中使用正確的類型。這讓我深刻了解到，在與資料庫的交互中，資料類型的精確匹配是非常重要的。

1. 堆疊溢出問題
在實現PartialEq時，我遇到了堆疊溢出的問題。我最初的實現包含了一個不小心的遞迴調用，導致了堆疊溢出。具體來說，在比較兩個枚舉值時，我不小心呼叫了自己的eq方法。透過更精確地處理每個變體，例如分別匹配每個錯誤變體並正確比較其內部值，我解決了這個問題。

1. 深入理解Rust模式匹配
透過上述的問題解決過程，我深入了解了Rust的模式匹配和如何使用它來比較枚舉的不同變體。我學會了如何避免遞迴引起的堆疊溢出，並如何更精確地處理我的匹配條件。

1. 實現FromRow以及query與query_as的使用差別
在開發過程中，我也遇到了需要使用query_as和實現FromRow的情況。query_as允許我將查詢結果映射到具體的Rust結構，而FromRow則是一個trait，允許自定義結構如何從資料庫行映射。與普通的query方法相比，query_as為查詢結果提供了更強的類型安全性。

## 08/11

### Title: fixed the unit test and refactor make_reservation function and create template reserve data.

finished the first stage of the reservation system, and I'm going to start the second stage. In the first stage, I've implemented the basic functions of the reservation system, including creating reservations, querying reservations, and handling conflicts. tomorrow I'll start the second stage, which is to check protobuf to make sure query function could be easy to use, so I'm going to implement pagination for database.

## 08/14

### Title: add new variable to the ReservationQuery, make sure I can implement pagination for database.

I discovered an issue in our database design, which made querying more cumbersome. To simplify the process, I added a new variable to the ReservationQuery, ensuring that I could implement pagination for the database. Today, I learned PL/pgSQL, a language for database programming, and began applying it to our system.

In the initial stage, I introduced new variables and implemented the 'rsvp.query' functions, becoming familiar with commands such as DECLARE, BEGIN, END, CASE END, RETURN, etc. I also learned how to use the quote_literal() function to prevent SQL injection, ensuring that incoming query statements correspond only to specific variables. This work has made our database queries more accessible and secure.

## 08/15

### Title: finished the query() update database function sql and refactor the validator, get_timespan

Today, I enhanced our protobuf by adding variables such as page, page_size, and desc for pagination purposes. I updated the database functions, got familiarized with SQL syntax, and implemented the query function. Along the way, I encountered the need to convert timestamps into PgRange timespans, prompting me to develop a dedicated function for it. Since validation was a frequent operation, I refactored it into a trait for broader usability across different .rs files. I also modularized the common checks for valid timespans, moving them to mod.rs. After a brief refactoring, I finished the query() function and wrote the corresponding tests.

## 08/16

### Title: familiar with builder pattern and wrote testcase for query function

Today, I refactored the ReservationQuery's new() method using the Builder design pattern. The previous version had too many parameters, which not only failed Clippy's checks but also felt uncomfortable to write. I employed the `derive_builder` crate to craft such a comprehensive function. While working on the tags, I found repetitive patterns, prompting me to use traits for modularity. After ensuring the previous tests passed, I added more tests for the query() function. I noticed that in Rust, both page and page_size are i32 types with a default value of 0. If unspecified, this leads to errors. So, I decided to handle this at the database layer to avoid any conflicts, regardless of the input. Finally, I tested functions related to PgRange.

## 08/17

### Title: added a new variable for cursor-based searching into gRPC to enhance database filter query functionality(1)

Today, I noticed a performance issue when querying our database. Using 'OFFSET' in SQL isn't efficient for large datasets since it has to go through all previous data. Instead, I'm exploring a method called 'key-set pagination'. This uses the ID as a sortable key and searches from a specific 'cursor' point using the primary key. before I take some action, I need to `dump the sql` in advance to make sure I don't do some silly things, after that, I've added parameters like 'prev' (previous), 'next', and 'is_desc' to our protobuf definition. After setting up the builder pattern to automate instance creation, I started writing the cursor logic. TO BE CONTINUED...

## 08/18

### Title: added a new variable for cursor-based searching into gRPC to enhance database filter query functionality(2)

Today, I continued working on the cursor logic, which is a bit more complicated than I expected. I've implemented the `keyset_pagination` function, and I realized we need a struct to hold pointers for the previous and next pages. I named it 'FilterPager', which lets users know if there are preceding or succeeding pages available.

## 08/19

### Title: review the keyset pagination

Today, I reviewed the keyset pagination and found that the previous implementation was incorrect. I've fixed the issue and added more tests to ensure the correctness of the code.

## 08/21

### Title: use tonic to combine that I crated function about reservation to gRPC(pre-configuration)

In the past few days, I've nearly completed the core functions of 'reservation'. Now, the goal is to integrate these features with the 'service' binary to form a gRPC interface. My first step was to implement the features defined in the protobuf's 'ReservationService' trait within the service. During this process, I realized I needed some configurations. So, I created 'config.rs' to manage 'DbConfig' and 'ServerConfig', wrapping them in a 'Config' for upcoming tasks like 'reserve', 'confirm', 'update', and more. I also wrote unit tests to ensure my config functions properly read the .yml files. Now, I'm diving into the main implementation. To be continued...

## 08/22

### Title: implement service main function

- [x] TODO: 就是在service做輸入和輸出，輸入成我們內部需要的參數和類型，並且接收回來處理後的資料，再轉換成gRPC的格式輸出。

I'm feeling a bit under the weather today. I'll lay out the details tomorrow. In short, I've implemented the core functions, and now I'm exposing them through a gRPC interface. Next, I'll set up a server to listen to incoming requests and respond accordingly.

## 08/22

### Title: chill out

I updated the code to have a more adaptable path and added documentation. Then, to make 'service/lib.rs' clearer, I split the implementation from the structure, placing them in 'service.rs' and 'lib.rs'.

## 08/28

### Title: calm down to think about how can I deal with the gRPC server interface's integration tests

Today, I finally clarified A TEST(ONE) that took me a while to understand. I began by diving deep into what 'runtime' is. Then, I pondered how to implement a synchronous 'Drop Trait' to drop an asynchronous database. However, I later chose a more direct approach. When testing, I first connected to the server and created a test database. Next, I connected to this database using its URL. This was followed by a series of gRPC server tests. After testing, it was important to drop the test database. But before doing so, I ensured all connections were closed.

## 08/29

### Title: the .env shouldn't be in the git, and use the git secrets instead.

I was unsure about including .env variables in my git repo. After researching, I learned about using git secrets for sensitive data. But during tests, I connect to a database through the 'sqlx_database_tester' crate, so I was unclear about its URL connection method. At first, I thought the URL from git secrets wasn't found. But the real issue was with parsing the URL. After changing 'postgres' to 'localhost', finally, I identified is the 'location(@)' problem.
