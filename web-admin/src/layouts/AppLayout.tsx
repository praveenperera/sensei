import { Outlet } from "react-router";
import AdminNav from "./AdminNav";

const AppLayout = () => {

  return (
    <>
      <AdminNav />

      <div className="md:pl-52 lg:pl-64 flex flex-col flex-1 pt-14">
        <main className="flex-1 overflow-y-auto bg-gray-background min-h-screen text-white">
          <div className="p-4 w-full max-w-5xl mx-auto">
            <Outlet />            
          </div>
        </main>
      </div>
    </>
  );
};

export default AppLayout;
