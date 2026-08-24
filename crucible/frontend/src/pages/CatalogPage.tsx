import React, { useState, useEffect } from 'react';
import { apiUrl } from '../lib/api';

interface Product {
  id: string;
  name: string;
  price: number;
  category?: string;
  in_stock?: boolean;
}

interface ProductListResponse {
  products: Product[];
  page: number;
  per_page: number;
  total_pages: number;
}

export const CatalogPage: React.FC = () => {
  const [products, setProducts] = useState<Product[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const controller = new AbortController();
    const loadCatalog = async () => {
      try {
        const res = await fetch(apiUrl('/products?page=1&per_page=12'), {
          headers: { 'X-Chaos': 'delay=5000ms' },
          signal: controller.signal,
        });
        if (!res.ok) {
          throw new Error(`HTTP error ${res.status}`);
        }
        const data: ProductListResponse = await res.json();
        if (active) {
          setProducts(data.products || []);
        }
      } catch (err: any) {
        if (err?.name === 'AbortError') {
          return;
        }
        if (active) {
          setErrorMessage('Failed to load product catalog.');
        }
      } finally {
        if (active) {
          setIsLoading(false);
        }
      }
    };
    loadCatalog();
    return () => {
      active = false;
      controller.abort();
    };
  }, []);

  return (
    <div className="page-container" data-testid="catalog-page">
      <div className="page-header">
        <span className="badge info">Network Layer: Response Stubbing &amp; Mocking</span>
        <h1>Product Catalog</h1>
        <p className="page-description">
          A paginated catalog backed by GET /products. Automation drills intercept this route to
          stub responses and verify rendering decoupled from backend availability.
        </p>
      </div>

      {isLoading && (
        <div className="catalog-loading" data-testid="catalog-loading">
          Loading catalog&hellip;
        </div>
      )}

      {errorMessage && <div className="alert-error">{errorMessage}</div>}

      {!isLoading && !errorMessage && (
        <div className="product-grid">
          {products.map((product) => (
            <div key={product.id} className="card product-card" data-testid="product-item">
              <div className="card-top">
                <span className={`badge ${product.in_stock === false ? 'danger' : 'info'}`}>
                  {product.in_stock === false ? 'Out of Stock' : 'In Stock'}
                </span>
              </div>
              <h3 className="product-name">{product.name}</h3>
              <div className="product-meta">
                <span className="product-price">${Number(product.price).toFixed(2)}</span>
                {product.category && (
                  <span className="product-category">{product.category}</span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
