use anyhow::Result;
use spy_core::Node;
use spy_storage::Storage;
use std::collections::HashMap;
use std::path::Path;

pub struct EmbeddingManager {
    storage: Storage,
}

impl EmbeddingManager {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn initialize_schema(&mut self) -> Result<()> {
        // Schema is already created in spy-storage migration
        Ok(())
    }

    pub fn generate_node_embeddings(&mut self, _model_path: &Path) -> Result<()> {
        // Simplified embedding generation using TF-IDF-like approach
        // This avoids heavy ML dependencies while still providing semantic search

        let nodes = self.storage.get_all_nodes()?;
        let total = nodes.len();

        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.storage.execute_raw(
            "INSERT INTO embedding_progress (total_nodes, processed_nodes, status, started_at) VALUES (?1, 0, 'running', ?2)",
            &[&(total as i32), &started_at],
        )?;

        for (i, node) in nodes.iter().enumerate() {
            // Generate simple keyword-based embedding
            let text = format!("{} {}", node.name, node.description.as_ref().unwrap_or(&String::new()));
            let embedding = self.generate_simple_embedding(&text);

            // Store embedding
            let node_id_str = node.node_id.as_str();
            self.storage.execute_raw(
                "INSERT OR REPLACE INTO node_embeddings (node_id, embedding, embedding_model, created_at) VALUES (?1, ?2, ?3, ?4)",
                &[&node_id_str, &embedding, &"simple-tfidf", &started_at],
            )?;

            // Update progress every 10 nodes
            if i % 10 == 0 {
                self.storage.execute_raw(
                    "UPDATE embedding_progress SET processed_nodes = ?1 WHERE id = (SELECT MAX(id) FROM embedding_progress)",
                    &[&(i as i32)],
                )?;
            }
        }

        let completed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.storage.execute_raw(
            "UPDATE embedding_progress SET processed_nodes = ?1, status = 'completed', completed_at = ?2 WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[&(total as i32), &completed_at],
        )?;

        Ok(())
    }

    fn generate_simple_embedding(&self, text: &str) -> Vec<u8> {
        // Simple TF-IDF-like embedding
        let mut word_counts: HashMap<String, u32> = HashMap::new();
        let mut total_words = 0;

        for word in text.split_whitespace() {
            let word = word.to_lowercase();
            *word_counts.entry(word).or_insert(0) += 1;
            total_words += 1;
        }

        // Create a fixed-size embedding vector (100 dimensions)
        let mut embedding = vec![0f32; 100];

        for (word, count) in word_counts {
            let tf = count as f32 / total_words as f32;
            let hash = self.hash_word(&word) as usize % 100;
            embedding[hash] += tf;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        // Convert to bytes
        let mut bytes = Vec::with_capacity(100 * 4);
        for val in embedding {
            bytes.extend_from_slice(&val.to_le_bytes());
        }

        bytes
    }

    fn hash_word(&self, word: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in word.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    pub fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<(Node, f64)>> {
        // Generate query embedding
        let query_embedding = self.generate_simple_embedding(query);
        let query_vec = self.bytes_to_vec(&query_embedding);

        // Fetch all embeddings
        let rows: Vec<(String, Vec<u8>)> = self.storage.query_raw(
            "SELECT node_id, embedding FROM node_embeddings",
            &[],
            |row| {
                let node_id: String = row.get(0)?;
                let embedding: Vec<u8> = row.get(1)?;
                Ok((node_id, embedding))
            },
        )?;

        let mut results = Vec::new();
        for (node_id, embedding_bytes) in rows {
            let embedding_vec = self.bytes_to_vec(&embedding_bytes);

            // Calculate cosine similarity
            let similarity = self.cosine_similarity(&query_vec, &embedding_vec);

            // Get node details
            if let Some(node) = self.storage.get_node(&node_id)? {
                results.push((node, similarity));
            }
        }

        // Sort by similarity and take top results
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    fn bytes_to_vec(&self, bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)) as f64
    }

    pub fn get_embedding_status(&self) -> Result<EmbeddingStatus> {
        let status: Option<String> = self.storage.query_row_raw(
            "SELECT status FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[],
            |row| row.get(0),
        )?;

        let total_nodes: i32 = self.storage.query_row_raw(
            "SELECT total_nodes FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[],
            |row| row.get(0),
        )?.unwrap_or(0);

        let processed_nodes: i32 = self.storage.query_row_raw(
            "SELECT processed_nodes FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[],
            |row| row.get(0),
        )?.unwrap_or(0);

        let started_at: i64 = self.storage.query_row_raw(
            "SELECT started_at FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[],
            |row| row.get(0),
        )?.unwrap_or(0);

        let completed_at: Option<i64> = self.storage.query_row_raw(
            "SELECT completed_at FROM embedding_progress WHERE id = (SELECT MAX(id) FROM embedding_progress)",
            &[],
            |row| row.get(0),
        )?;

        Ok(EmbeddingStatus {
            total_nodes,
            processed_nodes,
            status: status.unwrap_or_else(|| "not_started".to_string()),
            started_at,
            completed_at,
        })
    }
}

#[derive(Debug)]
pub struct EmbeddingStatus {
    pub total_nodes: i32,
    pub processed_nodes: i32,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}
