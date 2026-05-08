import { Navigate, Route, Routes } from 'react-router-dom';

import { EditorPage } from '@/pages/EditorPage';
import { HomePage } from '@/pages/HomePage';

export function AppRouter() {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/docs/:docId" element={<EditorPage />} />
      <Route path="*" element={<Navigate replace to="/" />} />
    </Routes>
  );
}
