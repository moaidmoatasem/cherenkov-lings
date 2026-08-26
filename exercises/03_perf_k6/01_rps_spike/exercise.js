import http from 'k6/http';
import { check } from 'k6';

export const options = {
  // TODO: Configure a spike test for the /checkout endpoint
};

export default function () {
  const res = http.get('http://localhost:8081/checkout');
  // TODO: Check that status is 200
}
