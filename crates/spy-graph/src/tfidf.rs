use spy_core::Node;
use std::collections::HashMap;

pub struct TfIdfIndex {
    nodes: Vec<Node>,
    idf: HashMap<String, f64>,
    tf_vectors: Vec<HashMap<String, f64>>,
    norms: Vec<f64>,
}

impl TfIdfIndex {
    pub fn build(nodes: Vec<Node>) -> Self {
        let mut doc_freq = HashMap::new();
        let mut tf_vectors = Vec::with_capacity(nodes.len());

        let num_docs = nodes.len() as f64;

        // Compute TF and DF
        for node in &nodes {
            let tokens = tokenize(node);
            let mut tf = HashMap::new();
            for token in &tokens {
                *tf.entry(token.clone()).or_insert(0.0) += 1.0;
            }

            let len = tokens.len() as f64;
            if len > 0.0 {
                for v in tf.values_mut() {
                    *v /= len;
                }
            }

            for token in tf.keys() {
                *doc_freq.entry(token.clone()).or_insert(0.0) += 1.0;
            }
            tf_vectors.push(tf);
        }

        // Compute IDF
        let mut idf = HashMap::new();
        for (token, df) in doc_freq {
            idf.insert(token, (num_docs / (1.0 + df)).ln() + 1.0);
        }

        // Compute TF-IDF vectors and their norms
        let mut norms = Vec::with_capacity(nodes.len());
        for tf in &mut tf_vectors {
            let mut norm_sq = 0.0;
            for (token, tf_val) in tf.iter_mut() {
                let idf_val = idf.get(token).unwrap_or(&1.0);
                let tfidf = *tf_val * idf_val;
                *tf_val = tfidf;
                norm_sq += tfidf * tfidf;
            }
            norms.push(norm_sq.sqrt());
        }

        Self {
            nodes,
            idf,
            tf_vectors,
            norms,
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<(Node, f64)> {
        let tokens = tokenize_text(query);
        let mut q_tf = HashMap::new();
        for token in &tokens {
            *q_tf.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        let len = tokens.len() as f64;
        if len > 0.0 {
            for v in q_tf.values_mut() {
                *v /= len;
            }
        }

        let mut q_norm_sq = 0.0;
        for (token, tf_val) in q_tf.iter_mut() {
            let idf_val = self.idf.get(token).unwrap_or(&1.0);
            let tfidf = *tf_val * idf_val;
            *tf_val = tfidf;
            q_norm_sq += tfidf * tfidf;
        }
        let q_norm = q_norm_sq.sqrt();

        if q_norm == 0.0 {
            return vec![];
        }

        let mut scores = Vec::with_capacity(self.nodes.len());
        for (i, tf) in self.tf_vectors.iter().enumerate() {
            let mut dot = 0.0;
            for (token, q_val) in &q_tf {
                if let Some(doc_val) = tf.get(token) {
                    dot += q_val * doc_val;
                }
            }

            let doc_norm = self.norms[i];
            let score = if doc_norm > 0.0 {
                dot / (q_norm * doc_norm)
            } else {
                0.0
            };

            if score > 0.0 {
                scores.push((self.nodes[i].clone(), score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);
        scores
    }
}

fn tokenize(node: &Node) -> Vec<String> {
    let mut text = node.name.clone();
    if let Some(desc) = &node.description {
        text.push(' ');
        text.push_str(desc);
    }
    for sig in &node.signatures {
        for param in &sig.params {
            text.push(' ');
            text.push_str(&param.name);
            if let Some(t) = &param.type_ {
                text.push(' ');
                text.push_str(t);
            }
        }
        if let Some(ret) = &sig.returns {
            text.push(' ');
            text.push_str(ret);
        }
    }
    tokenize_text(&text)
}

fn tokenize_text(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
