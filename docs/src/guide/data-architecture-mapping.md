# Data Architecture Mapping

## Overview

Data Architecture Mapping correlates database structure with code implementation to ensure queries, migrations, and business logic align correctly. This is critical for detecting missing indexes, foreign key misalignments, and partition/sharding issues.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│           Data Architecture Mapper                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. Schema Extraction                                   │
│     ├── Parse migrations                                │
│     ├── Extract ORM models                             │
│     └── Parse schema dumps                             │
│                                                          │
│  2. Entity Graph Construction                           │
│     ├── Tables/Collections                             │
│     ├── Relationships (FKs)                             │
│     ├── Indexes                                        │
│     ├── Partitions                                     │
│     └── Sharding keys                                  │
│                                                          │
│  3. Code-to-Entity Mapping                              │
│     ├── Query pattern analysis                         │
│     ├── Model usage tracking                           │
│     └── Repository pattern detection                   │
│                                                          │
│  4. Alignment Verification                             │
│     ├── Index coverage                                 │
│     ├── FK alignment                                   │
│     ├── Partition usage                                │
│     └── Sharding compliance                            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Schema Extraction

### Migration Parsing

**Laravel Example**:
```php
Schema::create('orders', function (Blueprint $table) {
    $table->id();
    $table->foreignId('user_id')->constrained();
    $table->decimal('total', 10, 2);
    $table->timestamps();
    
    $table->index('user_id');
    $table->index(['created_at', 'status']);
});
```

**Extracted Facts**:
- Table: `orders`
- Columns: `id`, `user_id`, `total`, `created_at`, `updated_at`
- Foreign Keys: `user_id` → `users.id`
- Indexes: `user_id`, `(created_at, status)`

### ORM Model Parsing

**Laravel Eloquent**:
```php
class Order extends Model {
    protected $fillable = ['user_id', 'total'];
    
    public function user() {
        return $this->belongsTo(User::class);
    }
}
```

**Extracted Facts**:
- Model: `Order`
- Table: `orders` (inferred)
- Relationships: `belongsTo(User)`
- Fillable fields: `user_id`, `total`

### Schema Dump Parsing

**PostgreSQL**:
```sql
CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id),
    total DECIMAL(10,2),
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_status ON orders(created_at, status);
```

**Extracted Facts**:
- Same as migration parsing
- Additional: Partition info, constraints, triggers

## Entity Graph Construction

### Graph Structure

```
┌─────────┐         ┌─────────┐
│  users  │◄────────│ orders  │
│         │         │         │
│  id (PK)│         │ id (PK) │
│  email  │         │user_id  │
│  name   │         │ total   │
└─────────┘         └─────────┘
     │                    │
     │                    │
     ▼                    ▼
┌─────────┐         ┌─────────┐
│profiles │         │order_   │
│         │         │items    │
│user_id  │         │order_id │
│(FK)     │         │(FK)     │
└─────────┘         └─────────┘
```

### Components

#### 1. Tables/Collections

```rust
struct Table {
    name: String,
    columns: Vec<Column>,
    primary_key: Option<Vec<String>>,
    indexes: Vec<Index>,
    foreign_keys: Vec<ForeignKey>,
    partitions: Option<PartitionInfo>,
    sharding_key: Option<String>,
}
```

#### 2. Relationships

```rust
struct ForeignKey {
    from_table: String,
    from_columns: Vec<String>,
    to_table: String,
    to_columns: Vec<String>,
    on_delete: CascadeAction,
    on_update: CascadeAction,
}
```

#### 3. Indexes

```rust
struct Index {
    name: String,
    table: String,
    columns: Vec<String>,
    unique: bool,
    partial: Option<String>, // WHERE clause for partial index
}
```

#### 4. Partitions

```rust
struct PartitionInfo {
    strategy: PartitionStrategy, // RANGE, LIST, HASH
    key_column: String,
    partitions: Vec<Partition>,
}
```

## Code-to-Entity Mapping

### Query Pattern Analysis

**Detect**:
- WHERE clauses
- JOIN patterns
- ORDER BY columns
- GROUP BY columns
- Aggregation functions

**Example**:
```php
Order::where('user_id', $userId)
     ->where('status', 'pending')
     ->orderBy('created_at', 'desc')
     ->get();
```

**Extracted Patterns**:
- WHERE: `user_id`, `status`
- ORDER BY: `created_at`
- Implicit JOIN: `users` (via `user_id` FK)

### Model Usage Tracking

**Track**:
- Model method calls
- Relationship access
- Query builder usage
- Raw SQL queries

### Repository Pattern Detection

**Detect**:
- Repository classes
- Data access patterns
- Query encapsulation

## Alignment Verification

### 1. Index Coverage

**Check**: Do WHERE clauses have corresponding indexes?

**Algorithm**:
1. Extract all WHERE clause columns
2. Check if index exists for column combination
3. Flag missing indexes

**Example**:
```php
// Query
Order::where('user_id', $id)
     ->where('status', 'pending')
     ->get();

// Required index
$table->index(['user_id', 'status']);

// Violation if missing
```

### 2. Foreign Key Alignment

**Check**: Do JOIN patterns match FK constraints?

**Algorithm**:
1. Extract JOIN patterns from queries
2. Verify FK exists for join columns
3. Check cascade actions match business logic

**Example**:
```php
// Query
Order::with('user')->get();

// Required FK
$table->foreign('user_id')
      ->references('id')
      ->on('users')
      ->onDelete('cascade');

// Violation if FK missing or cascade mismatch
```

### 3. Partition Usage

**Check**: Are partition keys used in WHERE clauses?

**Algorithm**:
1. Identify partitioned tables
2. Extract partition key column
3. Verify partition key appears in WHERE clauses

**Example**:
```sql
-- Table partitioned by created_at
CREATE TABLE orders (
    ...
) PARTITION BY RANGE (created_at);

-- Query must include partition key
SELECT * FROM orders 
WHERE created_at >= '2024-01-01'  -- ✓ Partition key used
  AND status = 'pending';

-- Violation if partition key missing
SELECT * FROM orders 
WHERE status = 'pending';  -- ✗ Missing partition key
```

### 4. Sharding Compliance

**Check**: Are sharding keys respected in queries?

**Algorithm**:
1. Identify sharded tables
2. Extract sharding key column
3. Verify sharding key in WHERE clauses
4. Flag cross-shard operations

**Example**:
```php
// Sharded by user_id
// Query must include user_id
Order::where('user_id', $userId)  // ✓ Sharding key present
     ->where('status', 'pending')
     ->get();

// Violation: Cross-shard query
Order::where('status', 'pending')  // ✗ Missing sharding key
     ->get();
```

## Product Correlation

### Entity Weighting

**Method**:
1. Extract domain entities from `product-metadata.json`
2. Map database entities to domain entities
3. Apply product-specific weighting

**Example**:
```json
{
  "product": {
    "type": "ecommerce"
  },
  "core_functionalities": [
    {
      "id": "order_processing",
      "name": "Order Processing",
      "criticality": "critical",
      "entry_points": [
        {"file": "app/Models/Order.php"},
        {"file": "app/Services/OrderService.php"}
      ]
    }
  ]
}
```

**Weighting**:
- `orders` table → `order_processing` functionality → `W_product = 1.3`
- Functions operating on `orders` → Higher critical weight

### Entity Centrality

**Calculate**:
- Number of relationships
- Query frequency
- Business importance

**Formula**:
```
Centrality = (
    0.4 * relationship_count_normalized +
    0.4 * query_frequency_normalized +
    0.2 * business_importance_score
)
```

## Implementation

### Schema Adapter Interface

```rust
trait SchemaAdapter {
    fn extract_tables(&self, source: &Path) -> Result<Vec<Table>>;
    fn extract_indexes(&self, source: &Path) -> Result<Vec<Index>>;
    fn extract_foreign_keys(&self, source: &Path) -> Result<Vec<ForeignKey>>;
    fn extract_partitions(&self, source: &Path) -> Result<Option<PartitionInfo>>;
}
```

### Framework Adapters

**Laravel Adapter**:
- Parse migration files
- Extract Eloquent models
- Parse schema dumps

**Django Adapter**:
- Parse migration files
- Extract Django models
- Parse schema dumps

**Rails Adapter**:
- Parse migration files
- Extract ActiveRecord models
- Parse schema.rb

### Query Analyzer

```rust
struct QueryAnalyzer {
    schema: EntityGraph,
    product_metadata: ProductMetadata,
}

impl QueryAnalyzer {
    fn analyze_query(&self, query: &Query) -> Vec<AlignmentIssue> {
        let mut issues = vec![];
        
        // Check index coverage
        if let Some(missing) = self.check_index_coverage(&query) {
            issues.push(missing);
        }
        
        // Check FK alignment
        if let Some(misaligned) = self.check_fk_alignment(&query) {
            issues.push(misaligned);
        }
        
        // Check partition usage
        if let Some(partition_issue) = self.check_partition_usage(&query) {
            issues.push(partition_issue);
        }
        
        // Check sharding compliance
        if let Some(sharding_issue) = self.check_sharding_compliance(&query) {
            issues.push(sharding_issue);
        }
        
        issues
    }
}
```

## Policy Integration

### Database Topology Policy

See `policies/laravel-db-topology.yaml` for complete policy definition.

**Key Checks**:
- Missing indexes for WHERE clauses
- Foreign key alignment with JOINs
- Partition key usage
- Sharding key compliance

## Best Practices

1. **Maintain Schema Graph**: Keep entity graph up-to-date with migrations
2. **Query Analysis**: Analyze all query patterns, not just ORM calls
3. **Product Metadata**: Maintain accurate domain entity mapping
4. **Regular Audits**: Run alignment checks in CI/CD
5. **Performance Monitoring**: Correlate slow queries with missing indexes

## Example Workflow

1. **Extract Schema**: Parse all migration files
2. **Build Graph**: Construct entity graph with relationships
3. **Analyze Queries**: Extract query patterns from code
4. **Verify Alignment**: Check indexes, FKs, partitions, sharding
5. **Report Issues**: Generate policy violations for misalignments
6. **Remediate**: Create migrations to fix issues

## Conclusion

Data Architecture Mapping ensures code and database structure remain aligned, preventing performance issues, data integrity problems, and scalability bottlenecks. By correlating schema with code patterns, the system can proactively detect and prevent common database-related issues.
